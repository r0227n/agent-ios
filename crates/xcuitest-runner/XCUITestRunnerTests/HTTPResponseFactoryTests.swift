import XCTest

final class HTTPResponseFactoryTests: XCTestCase {

    // MARK: - .ok() Factory

    func testOkWithNoBody() {
        let response = HTTPResponse.ok()
        XCTAssertEqual(response.statusCode, 200)
        XCTAssertNil(response.body)
    }

    func testOkWithDictionaryBody() {
        let response = HTTPResponse.ok(["status": "success"])
        XCTAssertEqual(response.statusCode, 200)
        let dict = response.body as? [String: String]
        XCTAssertEqual(dict?["status"], "success")
    }

    func testOkWithArrayBody() {
        let response = HTTPResponse.ok([1, 2, 3])
        XCTAssertEqual(response.statusCode, 200)
        let arr = response.body as? [Int]
        XCTAssertEqual(arr, [1, 2, 3])
    }

    // MARK: - .error() Factory

    func testErrorDefaultStatus400() {
        let response = HTTPResponse.error("bad input")
        XCTAssertEqual(response.statusCode, 400)
        let dict = response.body as? [String: String]
        XCTAssertEqual(dict?["error"], "bad input")
    }

    func testErrorWithCustomStatus() {
        let response = HTTPResponse.error("not found", status: 404)
        XCTAssertEqual(response.statusCode, 404)
        let dict = response.body as? [String: String]
        XCTAssertEqual(dict?["error"], "not found")
    }

    func testErrorWith500Status() {
        let response = HTTPResponse.error("internal failure", status: 500)
        XCTAssertEqual(response.statusCode, 500)
        let dict = response.body as? [String: String]
        XCTAssertEqual(dict?["error"], "internal failure")
    }
}
