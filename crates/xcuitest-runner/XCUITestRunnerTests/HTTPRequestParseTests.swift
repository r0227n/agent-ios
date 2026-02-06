import XCTest

final class HTTPRequestParseTests: XCTestCase {

    // MARK: - Successful Parsing

    func testParseSimpleGetRequest() {
        let raw = "GET /health HTTP/1.1\r\nHost: localhost\r\n\r\n"
        let request = HTTPRequest.parse(from: Data(raw.utf8))
        XCTAssertNotNil(request)
        XCTAssertEqual(request?.method, "GET")
        XCTAssertEqual(request?.path, "/health")
        XCTAssertTrue(request?.body.isEmpty ?? false)
    }

    func testParsePostRequestWithBody() {
        let body = "{\"x\":1,\"y\":2}"
        let raw = "POST /tap HTTP/1.1\r\nContent-Length: \(body.count)\r\n\r\n\(body)"
        let request = HTTPRequest.parse(from: Data(raw.utf8))
        XCTAssertNotNil(request)
        XCTAssertEqual(request?.method, "POST")
        XCTAssertEqual(request?.path, "/tap")
        XCTAssertEqual(request?.body, Data(body.utf8))
    }

    func testParseRequestWithMultipleHeaders() {
        let raw = "GET /info HTTP/1.1\r\nHost: localhost\r\nAccept: application/json\r\nX-Custom: value\r\n\r\n"
        let request = HTTPRequest.parse(from: Data(raw.utf8))
        XCTAssertNotNil(request)
        XCTAssertEqual(request?.headers.count, 3)
        XCTAssertEqual(request?.headers["host"], "localhost")
        XCTAssertEqual(request?.headers["accept"], "application/json")
        XCTAssertEqual(request?.headers["x-custom"], "value")
    }

    func testHeaderKeysAreLowercased() {
        let raw = "GET / HTTP/1.1\r\nContent-Type: text/plain\r\nX-REQUEST-ID: abc123\r\n\r\n"
        let request = HTTPRequest.parse(from: Data(raw.utf8))
        XCTAssertNotNil(request)
        XCTAssertEqual(request?.headers["content-type"], "text/plain")
        XCTAssertEqual(request?.headers["x-request-id"], "abc123")
    }

    func testParseRequestWithQueryString() {
        let raw = "GET /elements?bundleId=com.example.app HTTP/1.1\r\nHost: localhost\r\n\r\n"
        let request = HTTPRequest.parse(from: Data(raw.utf8))
        XCTAssertNotNil(request)
        XCTAssertEqual(request?.path, "/elements?bundleId=com.example.app")
    }

    func testParseUTF8MultibodyContent() {
        let body = "{\"text\":\"こんにちは世界\"}"
        let bodyData = Data(body.utf8)
        let header = "POST /type HTTP/1.1\r\nContent-Length: \(bodyData.count)\r\n\r\n"
        var data = Data(header.utf8)
        data.append(bodyData)
        let request = HTTPRequest.parse(from: data)
        XCTAssertNotNil(request)
        XCTAssertEqual(request?.body, bodyData)
    }

    func testParseGetRequestEmptyBody() {
        let raw = "GET /health HTTP/1.1\r\n\r\n"
        let request = HTTPRequest.parse(from: Data(raw.utf8))
        XCTAssertNotNil(request)
        XCTAssertTrue(request!.body.isEmpty)
    }

    func testParseRequestWithEmptyHeaderValue() {
        let raw = "GET / HTTP/1.1\r\nX-Empty: \r\n\r\n"
        let request = HTTPRequest.parse(from: Data(raw.utf8))
        XCTAssertNotNil(request)
        XCTAssertEqual(request?.headers["x-empty"], "")
    }

    // MARK: - Failure Cases

    func testParseEmptyDataReturnsNil() {
        let request = HTTPRequest.parse(from: Data())
        XCTAssertNil(request)
    }

    func testParseIncompleteHeadersReturnsNil() {
        let raw = "GET /health HTTP/1.1\r\nHost: localhost"
        let request = HTTPRequest.parse(from: Data(raw.utf8))
        XCTAssertNil(request, "Should return nil when \\r\\n\\r\\n separator is missing")
    }

    func testParseGarbageDataReturnsNil() {
        let data = Data([0xFF, 0xFE, 0x00, 0x01, 0x02])
        let request = HTTPRequest.parse(from: data)
        XCTAssertNil(request)
    }

    func testParseSingleByteReturnsNil() {
        let data = Data([0x47])  // 'G'
        let request = HTTPRequest.parse(from: data)
        XCTAssertNil(request)
    }

    func testParseOnlySeparatorReturnsNil() {
        let raw = "\r\n\r\n"
        let request = HTTPRequest.parse(from: Data(raw.utf8))
        // Request line is empty, parts.count < 2
        XCTAssertNil(request)
    }

    // MARK: - Edge Cases

    func testBodyContainingCRLFCRLF() {
        let bodyContent = "line1\r\n\r\nline2"
        let bodyData = Data(bodyContent.utf8)
        let header = "POST /data HTTP/1.1\r\nContent-Length: \(bodyData.count)\r\n\r\n"
        var data = Data(header.utf8)
        data.append(bodyData)
        let request = HTTPRequest.parse(from: data)
        XCTAssertNotNil(request)
        XCTAssertEqual(request?.body.count, bodyData.count)
        XCTAssertEqual(request?.body, bodyData)
    }

    func testLargeBody() {
        let bodyString = String(repeating: "A", count: 65536)
        let bodyData = Data(bodyString.utf8)
        let header = "POST /upload HTTP/1.1\r\nContent-Length: \(bodyData.count)\r\n\r\n"
        var data = Data(header.utf8)
        data.append(bodyData)
        let request = HTTPRequest.parse(from: data)
        XCTAssertNotNil(request)
        XCTAssertEqual(request?.body.count, 65536)
    }

    func testRequestLineWithExtraSpaces() {
        let raw = "GET /path HTTP/1.1 extra\r\n\r\n"
        let request = HTTPRequest.parse(from: Data(raw.utf8))
        XCTAssertNotNil(request, "Should parse even with extra parts in request line")
        XCTAssertEqual(request?.method, "GET")
        XCTAssertEqual(request?.path, "/path")
    }
}
