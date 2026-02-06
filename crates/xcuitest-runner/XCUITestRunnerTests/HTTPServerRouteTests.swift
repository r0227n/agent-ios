import Network
import XCTest

final class HTTPServerRouteTests: XCTestCase {

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

    private func startServer() throws -> UInt16 {
        server.get("/health") { _, completion in
            completion(.ok(["status": "ok"]))
        }
        server.post("/tap") { request, completion in
            let json = request.json() ?? [:]
            completion(.ok(["tapped": true, "x": json["x"] ?? 0, "y": json["y"] ?? 0]))
        }
        try server.start()
        // Wait for listener to be ready
        Thread.sleep(forTimeInterval: 0.3)
        guard let port = server.actualPort else {
            throw NSError(domain: "test", code: 1, userInfo: [NSLocalizedDescriptionKey: "Server port not available"])
        }
        return port
    }

    private func sendRequest(port: UInt16, method: String, path: String, body: String? = nil) throws -> (statusCode: Int, body: [String: Any]) {
        let semaphore = DispatchSemaphore(value: 0)
        var result: (Int, [String: Any]) = (0, [:])

        let connection = NWConnection(host: "127.0.0.1", port: NWEndpoint.Port(rawValue: port)!, using: .tcp)
        connection.start(queue: .global())

        let bodyData = body.flatMap { Data($0.utf8) } ?? Data()
        var raw = "\(method) \(path) HTTP/1.1\r\n"
        if !bodyData.isEmpty {
            raw += "Content-Length: \(bodyData.count)\r\n"
        }
        raw += "\r\n"
        var requestData = Data(raw.utf8)
        requestData.append(bodyData)

        connection.send(
            content: requestData,
            completion: .contentProcessed { error in
                if let error = error {
                    NSLog("Send error: \(error)")
                    semaphore.signal()
                    return
                }
                connection.receive(minimumIncompleteLength: 1, maximumLength: 65536) { data, _, _, _ in
                    if let data = data, let rawStr = String(data: data, encoding: .utf8) {
                        let parts = rawStr.components(separatedBy: "\r\n\r\n")
                        if let statusLine = parts.first?.components(separatedBy: "\r\n").first {
                            let statusParts = statusLine.components(separatedBy: " ")
                            if statusParts.count >= 2, let code = Int(statusParts[1]) {
                                result.0 = code
                            }
                        }
                        if parts.count > 1, let jsonData = parts[1].data(using: .utf8),
                            let json = try? JSONSerialization.jsonObject(with: jsonData) as? [String: Any]
                        {
                            result.1 = json
                        }
                    }
                    connection.cancel()
                    semaphore.signal()
                }
            })

        let waitResult = semaphore.wait(timeout: .now() + 5)
        if waitResult == .timedOut {
            connection.cancel()
            throw NSError(domain: "test", code: 2, userInfo: [NSLocalizedDescriptionKey: "Request timed out"])
        }
        return result
    }

    // MARK: - Route Tests

    func testGetRouteReturnsOK() throws {
        let port = try startServer()
        let (status, body) = try sendRequest(port: port, method: "GET", path: "/health")
        XCTAssertEqual(status, 200)
        XCTAssertEqual(body["status"] as? String, "ok")
    }

    func testPostRouteWithBody() throws {
        let port = try startServer()
        let (status, body) = try sendRequest(port: port, method: "POST", path: "/tap", body: "{\"x\":100,\"y\":200}")
        XCTAssertEqual(status, 200)
        XCTAssertEqual(body["tapped"] as? Bool, true)
    }

    func testUnregisteredRouteReturns404() throws {
        let port = try startServer()
        let (status, body) = try sendRequest(port: port, method: "GET", path: "/nonexistent")
        XCTAssertEqual(status, 404)
        XCTAssertNotNil(body["error"])
    }

    func testQueryStringStrippedForRouteMatching() throws {
        let port = try startServer()
        let (status, body) = try sendRequest(port: port, method: "GET", path: "/health?verbose=true")
        XCTAssertEqual(status, 200)
        XCTAssertEqual(body["status"] as? String, "ok")
    }

    func testWrongMethodReturns404() throws {
        let port = try startServer()
        let (status, _) = try sendRequest(port: port, method: "POST", path: "/health")
        XCTAssertEqual(status, 404, "POST to a GET-only route should return 404")
    }

    func testServerStopAndRestart() throws {
        let port1 = try startServer()
        let (status1, _) = try sendRequest(port: port1, method: "GET", path: "/health")
        XCTAssertEqual(status1, 200)

        server.stop()
        Thread.sleep(forTimeInterval: 0.3)

        // Restart with new port
        server = HTTPServer(port: 0)
        let port2 = try startServer()
        let (status2, _) = try sendRequest(port: port2, method: "GET", path: "/health")
        XCTAssertEqual(status2, 200)
    }
}
