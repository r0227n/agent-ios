import XCTest

final class HTTPResponseSerializeTests: XCTestCase {

    // MARK: - Helpers

    private func parseSerializedResponse(_ response: HTTPResponse) -> (statusLine: String, headers: [String: String], body: String) {
        let data = response.serialize()
        let raw = String(data: data, encoding: .utf8)!
        let parts = raw.components(separatedBy: "\r\n\r\n")
        let headerSection = parts[0]
        let bodySection = parts.count > 1 ? parts[1] : ""
        let headerLines = headerSection.components(separatedBy: "\r\n")
        let statusLine = headerLines[0]
        var headers: [String: String] = [:]
        for line in headerLines.dropFirst() {
            if let colonIndex = line.firstIndex(of: ":") {
                let key = String(line[..<colonIndex]).trimmingCharacters(in: .whitespaces)
                let value = String(line[line.index(after: colonIndex)...]).trimmingCharacters(in: .whitespaces)
                headers[key] = value
            }
        }
        return (statusLine, headers, bodySection)
    }

    // MARK: - Status Code Mapping

    func testSerialize200OK() {
        let (statusLine, _, _) = parseSerializedResponse(HTTPResponse(status: 200))
        XCTAssertEqual(statusLine, "HTTP/1.1 200 OK")
    }

    func testSerialize400BadRequest() {
        let (statusLine, _, _) = parseSerializedResponse(HTTPResponse(status: 400))
        XCTAssertEqual(statusLine, "HTTP/1.1 400 Bad Request")
    }

    func testSerialize404NotFound() {
        let (statusLine, _, _) = parseSerializedResponse(HTTPResponse(status: 404))
        XCTAssertEqual(statusLine, "HTTP/1.1 404 Not Found")
    }

    func testSerialize500InternalServerError() {
        let (statusLine, _, _) = parseSerializedResponse(HTTPResponse(status: 500))
        XCTAssertEqual(statusLine, "HTTP/1.1 500 Internal Server Error")
    }

    func testSerializeUnknownStatusCode() {
        let (statusLine, _, _) = parseSerializedResponse(HTTPResponse(status: 418))
        XCTAssertEqual(statusLine, "HTTP/1.1 418 Unknown")
    }

    // MARK: - Headers

    func testSerializeContentTypeIsJSON() {
        let (_, headers, _) = parseSerializedResponse(HTTPResponse(status: 200, body: ["key": "value"]))
        XCTAssertEqual(headers["Content-Type"], "application/json")
    }

    func testSerializeContentLengthMatchesBody() {
        let body: [String: Any] = ["key": "value"]
        let response = HTTPResponse(status: 200, body: body)
        let (_, headers, bodyStr) = parseSerializedResponse(response)
        let contentLength = Int(headers["Content-Length"] ?? "0")
        XCTAssertEqual(contentLength, bodyStr.utf8.count)
    }

    func testSerializeConnectionClose() {
        let (_, headers, _) = parseSerializedResponse(HTTPResponse(status: 200))
        XCTAssertEqual(headers["Connection"], "close")
    }

    // MARK: - Body

    func testSerializeNilBodyProducesEmptyJSON() {
        let (_, _, body) = parseSerializedResponse(HTTPResponse(status: 200))
        XCTAssertEqual(body, "{}")
    }
}
