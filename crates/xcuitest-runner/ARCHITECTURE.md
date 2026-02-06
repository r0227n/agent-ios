# XCUITest Runner - Architecture & Implementation Guide

## Overview

XCUITest Runner は、iOS Simulator 上で動作する軽量 HTTP サーバーです。XCUITest プロセス内で起動し、Rust CLI (`agent-mobile`) からの HTTP リクエストを受けて XCUITest API を呼び出します。

従来の idb gRPC + idb_companion アーキテクチャを置き換え、**外部依存ゼロ**（Apple 標準フレームワークのみ）で iOS 自動化を実現します。

## System Architecture

```mermaid
graph TB
    subgraph "agent-mobile (Rust CLI)"
        CLI[CLI Commands<br/>tap / swipe / snapshot ...]
        XCC[XCUITestClient<br/>reqwest HTTP]
        Helper["with_xcuitest()"]
        CLI --> Helper --> XCC
    end

    subgraph "iOS Simulator"
        subgraph "XCUITest Runner Process (xcodebuild test)"
            HTTP[HTTPServer<br/>NWListener :8200]
            AS[AutomationServer<br/>Route Registration]
            TH[TouchHandler]
            IH[InputHandler]
            AH[AccessibilityHandler]
            APH[AppHandler]

            HTTP --> AS
            AS --> TH
            AS --> IH
            AS --> AH
            AS --> APH
        end

        subgraph "Target App"
            UI[UI Elements]
            A11Y[Accessibility Tree]
        end

        TH -->|XCUICoordinate| UI
        IH -->|typeText / typeKey| UI
        AH -->|children matching| A11Y
        APH -->|launch / terminate| UI
    end

    XCC <-->|HTTP/JSON<br/>localhost:8200| HTTP

    subgraph "simctl (別経路)"
        SC[xcrun simctl]
    end

    CLI -.->|screenshot / clipboard<br/>boot / shutdown| SC
    SC -.-> UI
```

## Thread Model

XCUITest Runner の最も重要な設計上の制約は**スレッドモデル**です。

```mermaid
sequenceDiagram
    participant R as Rust CLI
    participant NW as NWListener<br/>(Background Queue)
    participant AS as AutomationServer<br/>(Route Handler)
    participant M as Main Thread<br/>(XCUITest API)

    R->>NW: HTTP POST /tap {"x":200,"y":400}
    NW->>AS: route(request, completion)
    AS->>M: DispatchQueue.main.async { work() }
    Note over M: XCUICoordinate.tap()
    M-->>AS: completion(HTTPResponse)
    AS-->>NW: sendResponse()
    NW-->>R: HTTP 200 {"success":true}
```

### Why Main Thread Dispatch?

| Component | Thread | Reason |
|-----------|--------|--------|
| NWListener | Background Queue | ネットワーク I/O はバックグラウンドで処理 |
| XCUITest API | **Main Thread Only** | Apple 制約: `XCUICoordinate.tap()` 等はメインスレッド必須 |
| RunLoop | Main Thread | テストを存続させつつメインスレッドのイベント処理を継続 |

**`onMain` ヘルパーパターン:**

```swift
private func onMain(
    _ work: @escaping () -> HTTPResponse,
    completion: @escaping (HTTPResponse) -> Void
) {
    DispatchQueue.main.async {
        let response = work()
        completion(response)
    }
}
```

> **Note:** `DispatchQueue.main.sync` は使用不可。メインスレッドが RunLoop で待機中のため、sync 呼び出しはデッドロックを引き起こします。

## Project Structure

```mermaid
graph LR
    subgraph "crates/xcuitest-runner/"
        subgraph "XCUITestRunner/ (Host App)"
            AD[AppDelegate.swift]
            IP1[Info.plist]
        end

        subgraph "XCUITestRunnerUITests/ (Test Bundle)"
            AS2[AutomationServer.swift]
            HS[HTTPServer.swift]
            TH2[TouchHandler.swift]
            IH2[InputHandler.swift]
            AH2[AccessibilityHandler.swift]
            APH2[AppHandler.swift]
            IP2[Info.plist]
        end

        XP[XCUITestRunner.xcodeproj]
    end

    subgraph "crates/platform-ios/src/xcuitest/"
        CL[client.rs]
        RN[runner.rs]
        TY[types.rs]
        MD[mod.rs]
    end

    subgraph "src/helpers/"
        HC[client.rs]
    end

    HC -->|uses| CL
    CL -->|HTTP| HS
    RN -->|xcodebuild test| XP
```

### File Roles

| File | Role | Layer |
|------|------|-------|
| `AutomationServer.swift` | テストエントリポイント + ルート登録 | Swift Server |
| `HTTPServer.swift` | NWListener ベース HTTP/1.1 サーバー | Swift Server |
| `TouchHandler.swift` | tap / swipe / long-press / button | Swift Handler |
| `InputHandler.swift` | text input / key press | Swift Handler |
| `AccessibilityHandler.swift` | Accessibility tree 取得 (idb 互換 JSON) | Swift Handler |
| `AppHandler.swift` | app launch / terminate | Swift Handler |
| `client.rs` | reqwest HTTP クライアント | Rust Client |
| `runner.rs` | xcodebuild プロセス管理 | Rust Client |
| `types.rs` | Request/Response serde 型定義 | Rust Client |
| `src/helpers/client.rs` | `with_xcuitest()` ヘルパー | Rust CLI |

## HTTP API Endpoints

```mermaid
graph LR
    subgraph "GET Endpoints"
        H["/health"]
        A["/accessibility"]
    end

    subgraph "POST Endpoints"
        T["/tap"]
        LP["/longpress"]
        S["/swipe"]
        TY2["/type"]
        KP["/keypress"]
        B["/button"]
        L["/launch"]
        TR["/terminate"]
        SA["/set-app"]
    end

    style H fill:#90EE90
    style A fill:#90EE90
    style T fill:#87CEEB
    style LP fill:#87CEEB
    style S fill:#87CEEB
    style TY2 fill:#87CEEB
    style KP fill:#87CEEB
    style B fill:#87CEEB
    style L fill:#FFD700
    style TR fill:#FFD700
    style SA fill:#FFD700
```

### Endpoint Details

#### Touch Operations

| Endpoint | Method | Body | XCUITest API |
|----------|--------|------|-------------|
| `/tap` | POST | `{"x": f64, "y": f64}` | `XCUICoordinate.tap()` |
| `/longpress` | POST | `{"x": f64, "y": f64, "duration": f64}` | `XCUICoordinate.press(forDuration:)` |
| `/swipe` | POST | `{"startX": f64, "startY": f64, "endX": f64, "endY": f64, "duration": f64}` | `coordinate.press(forDuration:thenDragTo:withVelocity:)` |

#### Input Operations

| Endpoint | Method | Body | XCUITest API |
|----------|--------|------|-------------|
| `/type` | POST | `{"text": "hello"}` | `XCUIElement.typeText()` |
| `/keypress` | POST | `{"key": "enter"}` | `XCUIElement.typeKey()` / `typeText()` |
| `/button` | POST | `{"button": "home"}` | `XCUIDevice.shared.press(.home)` |

**Key Mapping (`/keypress`):**

| Key Name | Mapping |
|----------|---------|
| `enter` / `return` | `\n` |
| `tab` | `\t` |
| `delete` / `backspace` | `.delete` |
| `escape` / `esc` | `.escape` |
| `space` | ` ` |
| `up` / `down` / `left` / `right` | Arrow keys |

#### Accessibility

| Endpoint | Method | Query Params | XCUITest API |
|----------|--------|-------------|-------------|
| `/accessibility` | GET | `?nested=true` (default) / `?nested=false` | Recursive `children(matching:)` traversal |

**Response Format (idb 互換):**

```json
{
  "type": "Button",
  "AXLabel": "Login",
  "AXValue": null,
  "frame": {"x": 100, "y": 200, "width": 80, "height": 44},
  "enabled": true,
  "children": []
}
```

#### App Management

| Endpoint | Method | Body | Effect |
|----------|--------|------|--------|
| `/launch` | POST | `{"bundleId": "com.apple.Settings"}` | Launch + handler context switch |
| `/terminate` | POST | `{"bundleId": "com.apple.Settings"}` | Terminate + revert to Springboard |
| `/set-app` | POST | `{"bundleId": "com.apple.Settings"}` | Context switch only (no launch) |

## Request/Response Lifecycle

```mermaid
flowchart TD
    A[TCP Connection] --> B[NWListener accepts]
    B --> C[receiveData accumulation]
    C --> D{HTTPRequest.parse<br/>successful?}
    D -->|No, not complete| C
    D -->|No, connection closed| E[400 Bad Request]
    D -->|Yes| F{Content-Length<br/>matches body?}
    F -->|Body incomplete| C
    F -->|Complete| G[Route Matching]
    G --> H{Path match?<br/>query string stripped}
    H -->|No match| I[404 Not Found]
    H -->|Match| J[onMain dispatch]
    J --> K[XCUITest API call<br/>on main thread]
    K --> L[Build HTTPResponse]
    L --> M[Serialize JSON + headers]
    M --> N[NWConnection.send]
    N --> O[Connection close]
```

## HTTP Request Parsing

HTTP リクエストのパースは**バイトレベル**で実装されています。これは `String.distance` が文字数を返すため、マルチバイト文字を含むリクエストでオフセット計算が狂う問題を回避するためです。

```mermaid
graph TD
    A[Raw TCP Data] --> B["Find \\r\\n\\r\\n separator<br/>(byte scan: 0x0D 0x0A 0x0D 0x0A)"]
    B --> C[Split at separator index]
    C --> D[Header bytes → UTF-8 String]
    C --> E[Body bytes → Data]
    D --> F[Parse request line<br/>METHOD PATH HTTP/1.1]
    D --> G[Parse headers<br/>key: value]
    G --> H[Extract Content-Length]
    H --> I{Body complete?}
    I -->|Yes| J[HTTPRequest ready]
    I -->|No| K[Continue accumulation]
```

**Implementation detail:**

```swift
// Byte-level separator search (NOT string-based)
let separator: [UInt8] = [0x0D, 0x0A, 0x0D, 0x0A]
for i in 0...(data.count - 4) {
    if data[data.startIndex + i] == separator[0]
        && data[data.startIndex + i + 1] == separator[1]
        && data[data.startIndex + i + 2] == separator[2]
        && data[data.startIndex + i + 3] == separator[3] {
        separatorIndex = i
        break
    }
}
```

## App Context Switching

```mermaid
stateDiagram-v2
    [*] --> Springboard: setUp()
    Springboard --> TargetApp: POST /launch
    TargetApp --> Springboard: POST /terminate
    TargetApp --> OtherApp: POST /launch (different bundleId)
    Springboard --> TargetApp: POST /set-app (no launch)

    note right of TargetApp
        All handlers updated:
        - TouchHandler(app: newApp)
        - InputHandler(app: newApp)
        - AccessibilityHandler(app: newApp)
    end note

    note right of Springboard
        Default context:
        com.apple.springboard
        Cross-app interaction enabled
    end note
```

## Rust Client Architecture

```mermaid
classDiagram
    class XCUITestClient {
        -base_url: String
        -http: reqwest::Client
        +new(port: u16) Self
        +default() Self
        +health_check() Result~bool~
        +wait_for_ready(timeout: Duration) Result
        +tap(x: f64, y: f64) Result
        +long_press(x: f64, y: f64, duration: f64) Result
        +swipe(start: tuple, end: tuple, duration: f64) Result
        +type_text(text: &str) Result
        +key_press(key: &str) Result
        +button_press(button: &str) Result
        +accessibility_info(nested: bool) Result~String~
        +launch_app(bundle_id: &str) Result
        +terminate_app(bundle_id: &str) Result
        +set_app(bundle_id: &str) Result
        -post~T~(path: &str, body: &T) Result
    }

    class XCUITestRunner {
        -project_path: PathBuf
        -destination: String
        -port: u16
        -process: Option~Child~
        +new(path: PathBuf, dest: String, port: u16) Self
        +bundled_project_path() Option~PathBuf~
        +start() Result
        +stop() Result
        +is_running() bool
        +client() XCUITestClient
    }

    class Types {
        <<serde structs>>
        TapRequest
        LongPressRequest
        SwipeRequest
        TypeTextRequest
        KeyPressRequest
        ButtonPressRequest
        AppRequest
        RunnerResponse
        HealthResponse
    }

    XCUITestRunner --> XCUITestClient : creates
    XCUITestClient --> Types : serializes/deserializes
```

### Timeout Strategy

```mermaid
graph LR
    subgraph "Default Client (30s)"
        T1[tap]
        T2[swipe]
        T3[type]
        T4[keypress]
        T5[longpress]
        T6[button]
        T7[launch]
        T8[terminate]
        T9[set-app]
    end

    subgraph "Dedicated Client (120s)"
        A1[accessibility_info]
    end

    subgraph "Connect Timeout (5s)"
        C[All endpoints]
    end

    style A1 fill:#FFB6C1
```

> Accessibility tree traversal は WebView を含むアプリで 60 秒以上かかるため、専用クライアント（120 秒タイムアウト）を使用します。

## Runner Lifecycle

```mermaid
sequenceDiagram
    participant CLI as agent-mobile
    participant Runner as XCUITestRunner (Rust)
    participant XB as xcodebuild
    participant Server as HTTPServer (Swift)

    CLI->>Runner: start()
    Runner->>XB: xcodebuild test-without-building<br/>-only-testing AutomationServer/testStartAutomationServer
    XB->>Server: setUp() + testStartAutomationServer()
    Server->>Server: NWListener.start(queue:)
    Server->>Server: RunLoop.run() (keep alive)

    loop Health Check (every 500ms, max 60s)
        Runner->>Server: GET /health
        Server-->>Runner: {"status":"ok"}
    end

    Runner-->>CLI: Ready!

    loop Normal Operation
        CLI->>Server: HTTP Request
        Server-->>CLI: HTTP Response
    end

    CLI->>Runner: stop()
    Runner->>XB: kill process
    XB->>Server: tearDown()
    Server->>Server: NWListener.cancel()
```

## Swipe Implementation Detail

スワイプは XCUITest の `press(forDuration:thenDragTo:withVelocity:)` を使用し、速度制御で精度を確保します。

```mermaid
graph TD
    A["swipe(startX, startY, endX, endY, duration)"] --> B[Calculate distance]
    B --> C["distance = sqrt((endX-startX)^2 + (endY-startY)^2)"]
    C --> D["velocity = max(distance / duration, 50)"]
    D --> E[Create start coordinate]
    D --> F[Create end coordinate]
    E --> G["start.press(forDuration: 0.05,<br/>thenDragTo: end,<br/>withVelocity: velocity,<br/>thenHoldForDuration: 0)"]
    F --> G
```

> `velocity` の最小値は 50 pts/sec。これより低いと XCUITest がスワイプを認識しない場合があります。

## Accessibility Tree Traversal

```mermaid
graph TD
    A[getAccessibilityTree] --> B{nested?}
    B -->|true| C[buildElementTree - recursive]
    B -->|false| D[buildFlatElement - root only]

    C --> E[buildElementDict]
    E --> F[type mapping<br/>70+ XCUIElement.ElementType]
    E --> G["Extract: AXLabel, AXValue,<br/>AXPlaceholderValue, frame, enabled"]
    C --> H[children matching .any]
    H --> I[Loop each child]
    I --> J{child.exists?}
    J -->|Yes| C
    J -->|No| K[Skip]

    style C fill:#FFB6C1
```

> **Performance Warning:** WebView を含むアプリでは、DOM 要素が Accessibility Tree に展開されるため、再帰走査が 60 秒以上かかる場合があります。`--no-scroll` オプションの使用を推奨します。

## Design Decisions

### Why NWListener (not GCDWebServer / Swifter)?

| Criteria | NWListener | GCDWebServer | Swifter |
|----------|-----------|-------------|---------|
| 外部依存 | **None** | CocoaPods/SPM | SPM |
| Apple 公式 | **Yes** | No | No |
| XCUITest 内動作 | **Verified** | Unverified | Unverified |
| HTTP/1.1 実装 | Manual | Built-in | Built-in |
| メンテナンス負担 | Medium | Low | Low |

**Decision:** 外部依存ゼロと Apple 公式フレームワークの安定性を優先。HTTP パースの手動実装コストは許容範囲内。

### Why HTTP/JSON (not gRPC / Unix Socket)?

| Criteria | HTTP/JSON | gRPC | Unix Socket |
|----------|----------|------|-------------|
| デバッグ容易性 | **curl で直接テスト可** | 専用ツール必要 | socat 等必要 |
| Rust 依存 | reqwest (軽量) | tonic+prost (重い) | tokio-uds |
| Swift 依存 | NWListener | SwiftGRPC | NWListener |
| パフォーマンス | Sufficient | Better | Better |
| 可読性 | **High** | Medium | Low |

**Decision:** デバッグ容易性と依存の軽さを優先。localhost 通信のため HTTP オーバーヘッドは無視できるレベル。

### Why Springboard as Default App?

```mermaid
graph LR
    SB[Springboard<br/>com.apple.springboard] -->|tap anywhere| Screen
    SB -->|access any app's UI| AnyApp
    SB -->|cross-app automation| MultiApp

    SA[Specific App] -->|only that app's UI| SingleApp
```

Springboard をデフォルトにすることで、`/launch` 前でもホーム画面操作やシステム UI との対話が可能になります。

## Known Issues & Limitations

| Issue | Severity | Workaround |
|-------|----------|------------|
| WebView accessibility traversal が遅い (60s+) | Medium | `--no-scroll` 使用、将来 `maxDepth` パラメータ追加予定 |
| Volume Up/Down ボタン非対応 | Low | Simulator では利用不可 |
| Lock/Siri ボタン非対応 | Low | XCUIDevice は `.home` のみ対応 |
| Runner 初回起動が遅い (xcodebuild) | Medium | 常駐プロセスとして運用 |
| Connection close per request | Low | HTTP Keep-Alive 未対応だがパフォーマンスへの影響は軽微 |

## Build & Run

### Prerequisites

- Xcode 16+ (iOS 16.0+ SDK)
- Booted iOS Simulator
- Rust toolchain (for agent-mobile CLI)

### Build XCUITest Runner

```bash
cd crates/xcuitest-runner
xcodebuild build-for-testing \
  -project XCUITestRunner.xcodeproj \
  -scheme XCUITestRunner \
  -destination 'platform=iOS Simulator,name=iPhone 16'
```

### Start Runner Manually

```bash
xcodebuild test-without-building \
  -project XCUITestRunner.xcodeproj \
  -scheme XCUITestRunner \
  -destination 'platform=iOS Simulator,name=iPhone 16' \
  -only-testing XCUITestRunnerUITests/AutomationServer/testStartAutomationServer
```

### Verify with curl

```bash
# Health check
curl http://localhost:8200/health

# Tap at (200, 400)
curl -X POST http://localhost:8200/tap \
  -H 'Content-Type: application/json' \
  -d '{"x": 200, "y": 400}'

# Get accessibility tree
curl http://localhost:8200/accessibility?nested=true
```
