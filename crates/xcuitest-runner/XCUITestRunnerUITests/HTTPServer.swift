import Foundation
import Network

/// Lightweight HTTP server using NWListener (no external dependencies).
/// Runs inside the XCUITest process on the simulator.
final class HTTPServer {
    typealias RouteHandler = (HTTPRequest, @escaping (HTTPResponse) -> Void) -> Void

    private var listener: NWListener?
    private var routes: [(method: String, path: String, handler: RouteHandler)] = []
    private let port: UInt16
    private let queue = DispatchQueue(label: "com.agent-mobile.xcuitest-server", qos: .userInteractive)

    var actualPort: UInt16? {
        listener?.port?.rawValue
    }

    init(port: UInt16 = 8200) {
        self.port = port
    }

    // MARK: - Route Registration

    /// Register a GET route handler.
    func get(_ path: String, handler: @escaping RouteHandler) {
        routes.append((method: "GET", path: path, handler: handler))
    }

    /// Register a POST route handler.
    func post(_ path: String, handler: @escaping RouteHandler) {
        routes.append((method: "POST", path: path, handler: handler))
    }

    // MARK: - Server Lifecycle

    /// Start the server and begin accepting requests on the configured port.
    func start() throws {
        let params = NWParameters.tcp
        params.allowLocalEndpointReuse = true

        let nwPort = NWEndpoint.Port(rawValue: port)!
        listener = try NWListener(using: params, on: nwPort)

        listener?.newConnectionHandler = { [weak self] connection in
            self?.handleConnection(connection)
        }

        listener?.stateUpdateHandler = { [weak self] state in
            guard let self = self else { return }
            switch state {
            case .ready:
                NSLog("[XCUITestRunner] HTTP server listening on port \(self.port)")
            case .failed(let error):
                NSLog("[XCUITestRunner] Server failed: \(error)")
            default:
                break
            }
        }

        listener?.start(queue: queue)
    }

    /// Stop the server and cancel the listener.
    func stop() {
        listener?.cancel()
        listener = nil
    }

    // MARK: - Connection Handling

    private func handleConnection(_ connection: NWConnection) {
        connection.start(queue: queue)
        receiveData(on: connection, accumulated: Data())
    }

    private func receiveData(on connection: NWConnection, accumulated: Data) {
        connection.receive(minimumIncompleteLength: 1, maximumLength: 65536) { [weak self] data, _, isComplete, error in
            guard let self = self else { return }

            if let error = error {
                NSLog("[XCUITestRunner] Receive error: \(error)")
                connection.cancel()
                return
            }

            var buffer = accumulated
            if let data = data {
                buffer.append(data)
            }

            // Try to parse the HTTP request
            if let request = HTTPRequest.parse(from: buffer) {
                // Check if we have the complete body
                if let contentLength = request.contentLength, request.body.count < contentLength {
                    if isComplete {
                        // Connection closed before body was complete
                        let response = HTTPResponse(status: 400, body: ["error": "Incomplete request body"])
                        self.sendResponse(response, on: connection)
                    } else {
                        // Need more data
                        self.receiveData(on: connection, accumulated: buffer)
                    }
                    return
                }

                self.route(request) { response in
                    self.sendResponse(response, on: connection)
                }
            } else if isComplete {
                let response = HTTPResponse(status: 400, body: ["error": "Invalid HTTP request"])
                self.sendResponse(response, on: connection)
            } else {
                // Need more data to parse headers
                self.receiveData(on: connection, accumulated: buffer)
            }
        }
    }

    private func route(_ request: HTTPRequest, completion: @escaping (HTTPResponse) -> Void) {
        // Strip query string for route matching
        let pathWithoutQuery = request.path.components(separatedBy: "?").first ?? request.path
        for route in routes {
            if route.method == request.method && route.path == pathWithoutQuery {
                route.handler(request, completion)
                return
            }
        }
        completion(HTTPResponse(status: 404, body: ["error": "Not found: \(request.method) \(request.path)"]))
    }

    private func sendResponse(_ response: HTTPResponse, on connection: NWConnection) {
        let data = response.serialize()
        connection.send(
            content: data,
            completion: .contentProcessed { error in
                if let error = error {
                    NSLog("[XCUITestRunner] Send error: \(error)")
                }
                connection.cancel()
            })
    }
}

// MARK: - HTTP Request

/// Represents a parsed HTTP request.
struct HTTPRequest {
    let method: String
    let path: String
    let headers: [String: String]
    let body: Data

    var contentLength: Int? {
        headers["content-length"].flatMap(Int.init)
    }

    /// Parse JSON body into a dictionary.
    func json() -> [String: Any]? {
        guard !body.isEmpty else { return nil }
        return try? JSONSerialization.jsonObject(with: body) as? [String: Any]
    }

    /// Parse HTTP request from raw data.
    static func parse(from data: Data) -> HTTPRequest? {
        // Find header/body separator (\r\n\r\n) in raw bytes
        let separator: [UInt8] = [0x0D, 0x0A, 0x0D, 0x0A]  // \r\n\r\n
        var separatorIndex: Int? = nil
        if data.count >= 4 {
            for i in 0...(data.count - 4) {
                if data[data.startIndex + i] == separator[0]
                    && data[data.startIndex + i + 1] == separator[1]
                    && data[data.startIndex + i + 2] == separator[2]
                    && data[data.startIndex + i + 3] == separator[3]
                {
                    separatorIndex = i
                    break
                }
            }
        }
        guard let sepIdx = separatorIndex else { return nil }

        let headerData = data.subdata(in: data.startIndex..<data.startIndex.advanced(by: sepIdx))
        let bodyStart = data.startIndex.advanced(by: sepIdx + 4)
        let bodyData = data.subdata(in: bodyStart..<data.endIndex)

        guard let headerPart = String(data: headerData, encoding: .utf8) else { return nil }

        let lines = headerPart.components(separatedBy: "\r\n")
        guard let requestLine = lines.first else { return nil }

        let parts = requestLine.components(separatedBy: " ")
        guard parts.count >= 2 else { return nil }

        let method = parts[0]
        let path = parts[1]

        var headers: [String: String] = [:]
        for line in lines.dropFirst() {
            if let colonIndex = line.firstIndex(of: ":") {
                let key = String(line[line.startIndex..<colonIndex]).trimmingCharacters(in: .whitespaces).lowercased()
                let value = String(line[line.index(after: colonIndex)...]).trimmingCharacters(in: .whitespaces)
                headers[key] = value
            }
        }

        return HTTPRequest(method: method, path: path, headers: headers, body: bodyData)
    }
}

// MARK: - HTTP Response

/// Represents an HTTP response to be sent back to the client.
struct HTTPResponse {
    let statusCode: Int
    let body: Any?
    /// Optional raw binary data with custom content type (overrides JSON body).
    private let rawData: Data?
    private let contentType: String

    /// Create a JSON response.
    init(status: Int, body: Any? = nil) {
        self.statusCode = status
        self.body = body
        self.rawData = nil
        self.contentType = "application/json"
    }

    /// Create a response with raw binary data and a custom content type.
    init(status: Int, data: Data, contentType: String) {
        self.statusCode = status
        self.body = nil
        self.rawData = data
        self.contentType = contentType
    }

    /// Create a 200 OK response.
    static func ok(_ body: Any? = nil) -> HTTPResponse {
        HTTPResponse(status: 200, body: body)
    }

    /// Create an error response.
    static func error(_ message: String, status: Int = 400) -> HTTPResponse {
        HTTPResponse(status: status, body: ["error": message])
    }

    /// Create a binary data response (e.g. PNG image).
    static func data(_ data: Data, contentType: String, status: Int = 200) -> HTTPResponse {
        HTTPResponse(status: status, data: data, contentType: contentType)
    }

    /// Serialize the response into raw HTTP bytes.
    func serialize() -> Data {
        let bodyData: Data
        if let raw = rawData {
            bodyData = raw
        } else if let body = body {
            bodyData = (try? JSONSerialization.data(withJSONObject: body, options: [.sortedKeys])) ?? Data()
        } else {
            bodyData = "{}".data(using: .utf8)!
        }

        let statusText: String
        switch statusCode {
        case 200: statusText = "OK"
        case 400: statusText = "Bad Request"
        case 404: statusText = "Not Found"
        case 500: statusText = "Internal Server Error"
        default: statusText = "Unknown"
        }

        let header =
            "HTTP/1.1 \(statusCode) \(statusText)\r\nContent-Type: \(contentType)\r\nContent-Length: \(bodyData.count)\r\nConnection: close\r\n\r\n"

        var result = header.data(using: .utf8)!
        result.append(bodyData)
        return result
    }
}
