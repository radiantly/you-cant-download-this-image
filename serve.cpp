#include <arpa/inet.h>
#include <errno.h>
#include <netdb.h>
#include <netinet/in.h>
#include <signal.h>
#include <string.h>
#include <sys/socket.h>
#include <sys/types.h>
#include <sys/wait.h>
#include <unistd.h>

#include <chrono>
#include <cstring>
#include <fstream>
#include <iomanip>
#include <iostream>
#include <mutex>
#include <queue>
#include <sstream>
#include <string>
#include <thread>
#include <vector>

#define PORT "3000"
#define BACKLOG 10

std::string toHex(const int num) {
    std::stringstream stream;
    stream << std::hex << num;
    return stream.str();
}

class client_info {
    using timepoint = std::chrono::time_point<std::chrono::steady_clock>;

   public:
    int sockfd;
    timepoint last_updated;

    client_info(int fd, timepoint t) : sockfd(fd), last_updated(t){};
};

std::vector<int> clientfds;
std::mutex fd_queue_mutex;

int sendall(const int sockfd, const std::string& data) {
    int bytes_sent{}, result{};
    while (bytes_sent < data.size()) {
        result = send(sockfd, data.data() + bytes_sent, data.size() - bytes_sent, MSG_NOSIGNAL);
        if (result <= 0) break;
        bytes_sent -= result;
    }
    return result;
}

bool inline process(const int sockfd) {
    static const std::string data = "1\r\nf\r\n";
    return sendall(sockfd, data) > 0;
}

void keep_connections_open() {
    using namespace std::chrono;

    std::queue<client_info> clients;
    while (true) {
        const auto now = steady_clock::now();
        if (!clientfds.empty()) {
            std::scoped_lock lock(fd_queue_mutex);
            for (auto& clientfd : clientfds) {
                clients.emplace(clientfd, now);
            }
            std::cerr << "Added " << clientfds.size() << " client to queue. Total client count: " << clients.size() << ".\n";
            clientfds.clear();
        }

        if (clients.empty()) {
            std::this_thread::sleep_for(10s);
            continue;
        }

        if (clients.front().last_updated + 10s > now) {
            std::this_thread::sleep_until(clients.front().last_updated + 10s);
        }

        bool processed = process(clients.front().sockfd);
        if (processed) {
            clients.emplace(clients.front().sockfd, now);
        } else {
            close(clients.front().sockfd);
        }
        clients.pop();
    }
}

std::string readFile(std::string filename) {
    std::ifstream file(filename, std::ios::binary);
    std::string content((std::istreambuf_iterator<char>(file)),
                        (std::istreambuf_iterator<char>()));

    return content;
}

std::string buildResponse() {
    std::string response =
        "HTTP/1.1 200 OK\r\n"
        "Content-Type: image/jpeg\r\n"
        "Cache-Control: no-store\r\n"
        "X-Accel-Buffering: no\r\n"
        "Transfer-Encoding: chunked\r\n"
        "\r\n";

    auto image = readFile("public/lisa.jpg");
    response += toHex(image.size());
    response += "\r\n";
    response += image;
    response += "\r\n";

    return response;
}

int main() {
    std::cout << "Starting server..\n";

    int status, serverfd;
    addrinfo hints, *res;

    memset(&hints, 0, sizeof(hints));
    hints.ai_family = AF_UNSPEC;
    hints.ai_socktype = SOCK_STREAM;
    hints.ai_flags = AI_PASSIVE;

    if (status = getaddrinfo(NULL, PORT, &hints, &res)) {
        std::cout << "getaddrinfo error " << gai_strerror(status) << std::endl;
        exit(1);
    }

    if ((serverfd = socket(res->ai_family, res->ai_socktype, res->ai_protocol)) == -1) {
        perror("server: socket");
        exit(1);
    }

    if (bind(serverfd, res->ai_addr, res->ai_addrlen) == -1) {
        perror("server: bind");
        exit(1);
    }

    if (listen(serverfd, BACKLOG) == -1) {
        perror("server: listen");
        exit(1);
    }

    std::thread t(keep_connections_open);

    auto response = buildResponse();

    sockaddr_storage client_addr;
    socklen_t sin_size;
    while (true) {
        sin_size = sizeof(client_addr);
        int clientfd = accept(serverfd, reinterpret_cast<sockaddr*>(&client_addr), &sin_size);
        if (clientfd == -1) {
            perror("server: accept");
            continue;
        }

        if (sendall(clientfd, response) > 0) {
            std::scoped_lock lock(fd_queue_mutex);
            clientfds.push_back(clientfd);
        } else {
            close(clientfd);
        }
    }

    freeaddrinfo(res);

    return 0;
}