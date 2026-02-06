import XCTest
import Network

/// Integration tests that verify the HTTP protocol contract between
/// the XCUITest Runner and the Rust XCUITestClient.
final class HTTPServerIntegrationTests: XCTestCase {

    private var server: HTTPServer!

    override func setUp() {
        super.setUp()
        server = HTTPServer(port: 0)
    }

    override func tearDown() {
        server.stop()
        server = nil
        super.tearDown()
    }

    // MARK: - Helpers

    private func startServer(routes: (HTTPServer) -> Void) throws -> UInt16 {
        routes(server)
        try server.start()
        Thread.sleep(forTimeInterval: 0.3)
        guard let port = server.actualPort else {
            throw NSError(domain: "test", code: 1, userInfo: [NSLocalizedDescriptionKey: "Server port not available"])
        }
        return port
    }

    private func sendRawRequest(port: UInt16, raw: Data) throws -> Data {
        let semaphore = DispatchSemaphore(value: 0)
        var responseData = Data()

        let connection = NWConnection(host: "127.0.0.1", port: NWEndpoint.Port(rawValue: port)!, using: .tcp)
        connection.start(queue: .global())

        connection.send(content: raw, completion: .contentProcessed { error in
            if let error = error {
                NSLog("Send error: \(error)")
                semaphore.signal()
                return
            }
            connection.receive(minimumIncompleteLength: 1, maximumLength: 65536) { data, _, _, _ in
                if let data = data {
                    responseData = data
                }
                connection.cancel()
                semaphore.signal()
            }
        })

        let result = semaphore.wait(timeout: .now() + 5)
        if result == .timedOut {
            connection.cancel()
            throw NSError(domain: "test", code: 2, userInfo: [NSLocalizedDescriptionKey: "Request timed out"])
        }
        return responseData
    }

    private func parseHTTPResponse(_ data: Data) -> (statusCode: Int, headers: [String: String], body: [String: Any]) {
        guard let raw = String(data: data, encoding: .utf8) else { return (0, [:], [:]) }
        let parts = raw.components(separatedBy: "\r\n\r\n")
        guard let headerSection = parts.first else { return (0, [:], [:]) }
        let headerLines = headerSection.components(separatedBy: "\r\n")

        var statusCode = 0
        if let statusLine = headerLines.first {
            let statusParts = statusLine.components(separatedBy: " ")
            if statusParts.count >= 2, let code = Int(statusParts[1]) {
                statusCode = code
            }
        }

        var headers: [String: String] = [:]
        for line in headerLines.dropFirst() {
            if let colonIdx = line.firstIndex(of: ":") {
                let key = String(line[..<colonIdx]).trimmingCharacters(in: .whitespaces)
                let value = String(line[line.index(after: colonIdx)...]).trimmingCharacters(in: .whitespaces)
                headers[key] = value
            }
        }

        var body: [String: Any] = [:]
        if parts.count > 1, let jsonData = parts[1].data(using: .utf8),
           let json = try? JSONSerialization.jsonObject(with: jsonData) as? [String: Any] {
            body = json
        }

        return (statusCode, headers, body)
    }

    // MARK: - Protocol Contract Tests

    func testResponseHasCorrectHTTPVersion() throws {
        let port = try startServer { $0.get("/ping") { _, c in c(.ok(["pong": true])) } }
        let raw = "GET /ping HTTP/1.1\r\n\r\n"
        let data = try sendRawRequest(port: port, raw: Data(raw.utf8))
        let rawStr = String(data: data, encoding: .utf8) ?? ""
        XCTAssertTrue(rawStr.hasPrefix("HTTP/1.1"), "Response should use HTTP/1.1")
    }

    func testResponseContentTypeIsJSON() throws {
        let port = try startServer { $0.get("/ping") { _, c in c(.ok(["pong": true])) } }
        let raw = "GET /ping HTTP/1.1\r\n\r\n"
        let data = try sendRawRequest(port: port, raw: Data(raw.utf8))
        let (_, headers, _) = parseHTTPResponse(data)
        XCTAssertEqual(headers["Content-Type"], "application/json")
    }

    func testResponseContentLengthIsAccurate() throws {
        let port = try startServer { $0.get("/data") { _, c in c(.ok(["key": "value"])) } }
        let raw = "GET /data HTTP/1.1\r\n\r\n"
        let data = try sendRawRequest(port: port, raw: Data(raw.utf8))
        let (_, headers, _) = parseHTTPResponse(data)
        let rawStr = String(data: data, encoding: .utf8) ?? ""
        let parts = rawStr.components(separatedBy: "\r\n\r\n")
        let bodyBytes = parts.count > 1 ? parts[1].utf8.count : 0
        let contentLength = Int(headers["Content-Length"] ?? "0") ?? 0
        XCTAssertEqual(contentLength, bodyBytes)
    }

    func testResponseConnectionCloseHeader() throws {
        let port = try startServer { $0.get("/ping") { _, c in c(.ok()) } }
        let raw = "GET /ping HTTP/1.1\r\n\r\n"
        let data = try sendRawRequest(port: port, raw: Data(raw.utf8))
        let (_, headers, _) = parseHTTPResponse(data)
        XCTAssertEqual(headers["Connection"], "close")
    }

    func testSortedJSONKeys() throws {
        let port = try startServer {
            $0.get("/sorted") { _, c in c(.ok(["z_last": 1, "a_first": 2, "m_middle": 3])) }
        }
        let raw = "GET /sorted HTTP/1.1\r\n\r\n"
        let data = try sendRawRequest(port: port, raw: Data(raw.utf8))
        let rawStr = String(data: data, encoding: .utf8) ?? ""
        let parts = rawStr.components(separatedBy: "\r\n\r\n")
        guard parts.count > 1 else { XCTFail("No body"); return }
        let bodyStr = parts[1]
        // With .sortedKeys, "a_first" should appear before "m_middle" which should appear before "z_last"
        if let aRange = bodyStr.range(of: "a_first"),
           let mRange = bodyStr.range(of: "m_middle"),
           let zRange = bodyStr.range(of: "z_last") {
            XCTAssertTrue(aRange.lowerBound < mRange.lowerBound)
            XCTAssertTrue(mRange.lowerBound < zRange.lowerBound)
        } else {
            XCTFail("Expected keys not found in response body: \(bodyStr)")
        }
    }

    func testErrorResponseFormat() throws {
        let port = try startServer { $0.get("/ping") { _, c in c(.ok()) } }
        let raw = "GET /missing HTTP/1.1\r\n\r\n"
        let data = try sendRawRequest(port: port, raw: Data(raw.utf8))
        let (statusCode, _, body) = parseHTTPResponse(data)
        XCTAssertEqual(statusCode, 404)
        XCTAssertNotNil(body["error"] as? String)
    }

    func testConcurrentRequests() throws {
        let port = try startServer {
            $0.get("/id") { request, completion in
                let id = request.path.components(separatedBy: "?id=").last ?? "unknown"
                completion(.ok(["id": id]))
            }
        }

        let group = DispatchGroup()
        let count = 10
        var results = [Int](repeating: 0, count: count)

        for i in 0..<count {
            group.enter()
            DispatchQueue.global().async { [self] in
                defer { group.leave() }
                let raw = "GET /id?id=\(i) HTTP/1.1\r\n\r\n"
                if let data = try? self.sendRawRequest(port: port, raw: Data(raw.utf8)) {
                    let (statusCode, _, _) = self.parseHTTPResponse(data)
                    results[i] = statusCode
                }
            }
        }

        let groupResult = group.wait(timeout: .now() + 10)
        XCTAssertEqual(groupResult, .success)
        for i in 0..<count {
            XCTAssertEqual(results[i], 200, "Request \(i) should succeed")
        }
    }

    func testLargeJSONResponseBody() throws {
        let largeValue = String(repeating: "x", count: 10000)
        let port = try startServer {
            $0.get("/large") { _, c in c(.ok(["data": largeValue])) }
        }
        let raw = "GET /large HTTP/1.1\r\n\r\n"
        let data = try sendRawRequest(port: port, raw: Data(raw.utf8))
        let (statusCode, headers, body) = parseHTTPResponse(data)
        XCTAssertEqual(statusCode, 200)
        let contentLength = Int(headers["Content-Length"] ?? "0") ?? 0
        XCTAssertGreaterThan(contentLength, 10000)
        XCTAssertEqual((body["data"] as? String)?.count, 10000)
    }
}
