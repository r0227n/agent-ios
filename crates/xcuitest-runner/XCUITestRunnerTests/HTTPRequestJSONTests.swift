import XCTest

final class HTTPRequestJSONTests: XCTestCase {

    // MARK: - Helpers

    private func makeRequest(body: String) -> HTTPRequest {
        let bodyData = Data(body.utf8)
        let raw = "POST /test HTTP/1.1\r\nContent-Length: \(bodyData.count)\r\n\r\n"
        var data = Data(raw.utf8)
        data.append(bodyData)
        return HTTPRequest.parse(from: data)!
    }

    private func makeRequest(bodyData: Data, contentLength: Int? = nil) -> HTTPRequest {
        let cl = contentLength ?? bodyData.count
        let raw = "POST /test HTTP/1.1\r\nContent-Length: \(cl)\r\n\r\n"
        var data = Data(raw.utf8)
        data.append(bodyData)
        return HTTPRequest.parse(from: data)!
    }

    // MARK: - json() Tests

    func testJsonParsesValidObject() {
        let request = makeRequest(body: "{\"x\":100,\"y\":200}")
        let json = request.json()
        XCTAssertNotNil(json)
        XCTAssertEqual(json?["x"] as? Int, 100)
        XCTAssertEqual(json?["y"] as? Int, 200)
    }

    func testJsonParsesNestedObject() {
        let request = makeRequest(body: "{\"point\":{\"x\":1,\"y\":2}}")
        let json = request.json()
        XCTAssertNotNil(json)
        let point = json?["point"] as? [String: Any]
        XCTAssertNotNil(point)
        XCTAssertEqual(point?["x"] as? Int, 1)
    }

    func testJsonParsesStringValues() {
        let request = makeRequest(body: "{\"bundleId\":\"com.example.app\"}")
        let json = request.json()
        XCTAssertEqual(json?["bundleId"] as? String, "com.example.app")
    }

    func testJsonParsesBooleanValues() {
        let request = makeRequest(body: "{\"enabled\":true}")
        let json = request.json()
        XCTAssertEqual(json?["enabled"] as? Bool, true)
    }

    func testJsonParsesArrayValue() {
        let request = makeRequest(body: "{\"ids\":[1,2,3]}")
        let json = request.json()
        let ids = json?["ids"] as? [Int]
        XCTAssertEqual(ids, [1, 2, 3])
    }

    func testJsonReturnsNilForEmptyBody() {
        let raw = "GET /health HTTP/1.1\r\n\r\n"
        let request = HTTPRequest.parse(from: Data(raw.utf8))!
        XCTAssertNil(request.json())
    }

    func testJsonReturnsNilForInvalidJSON() {
        let request = makeRequest(body: "not json at all")
        XCTAssertNil(request.json())
    }

    func testJsonReturnsNilForArrayRoot() {
        let request = makeRequest(body: "[1,2,3]")
        XCTAssertNil(request.json(), "json() expects [String: Any] dictionary, not array")
    }

    // MARK: - contentLength Tests

    func testContentLengthParsedFromHeader() {
        let request = makeRequest(body: "{\"a\":1}")
        XCTAssertEqual(request.contentLength, 7)
    }

    func testContentLengthNilWhenMissing() {
        let raw = "GET /health HTTP/1.1\r\nHost: localhost\r\n\r\n"
        let request = HTTPRequest.parse(from: Data(raw.utf8))!
        XCTAssertNil(request.contentLength)
    }
}
