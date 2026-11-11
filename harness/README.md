# lai-harness

Trợ lý AI cá nhân chạy trên máy của bạn (macOS, Windows, Linux). Agent nhận việc qua dòng lệnh, tự kiểm tra Gmail theo lịch và viết báo cáo. Mọi hành động thay đổi bên ngoài (gửi thư...) đều phải có người duyệt.

Trạng thái: bản dòng lệnh. Chưa có app giao diện và file cài đặt; xem [Chưa làm](#chưa-làm).

## Cài đặt

Cần [Rust](https://rustup.rs).

```sh
cd harness
cargo install --path .
lai init        # tạo file cấu hình mẫu
```

`lai init` in ra đường dẫn file `config.toml`. Toàn bộ agent, provider và lịch chạy khai báo trong file đó.

## Chọn "bộ não": API key hoặc gói subscription

Mỗi agent trỏ tới một provider trong `config.toml`.

| Provider | Dùng gì | Thiết lập |
|---|---|---|
| `kind = "anthropic"` | API key, trả theo lượng dùng | `lai auth anthropic` |
| `kind = "claude-cli"` | Gói Claude đã đăng nhập trong Claude Code | Cài [Claude Code](https://claude.com/claude-code), chạy `claude` và đăng nhập một lần |

Với `claude-cli`, lai chỉ gọi lệnh `claude` chính thức; lai không đọc và không lưu token đăng nhập. Các công cụ có sẵn của Claude Code (chạy lệnh, sửa file, web) bị tắt, agent chỉ dùng được công cụ khai báo trong `config.toml`.

Lưu ý về gói subscription: gói cá nhân dành cho một người dùng trực tiếp. Chạy tự động liên tục ở chế độ nền có thể nằm ngoài phạm vi đó; hãy đọc điều khoản của Anthropic trước khi cho job chạy dày, và dùng API key cho việc chạy nền nặng.

## Kết nối Gmail

lai dùng Gmail API với quyền đọc thư và gửi thư. Mỗi người tự tạo OAuth client của mình, một lần:

1. Vào [Google Cloud Console](https://console.cloud.google.com), tạo project, bật **Gmail API**.
2. Cấu hình OAuth consent screen (loại External), thêm chính email của bạn vào Test users.
3. Tạo Credentials → OAuth client ID → loại **Desktop app**. Lấy Client ID và Client secret.
4. Chạy `lai auth gmail`, dán hai giá trị đó, đăng nhập trên trình duyệt.

Khi consent screen còn ở trạng thái Testing, Google thường thu hồi đăng nhập sau khoảng 7 ngày và bạn phải chạy lại `lai auth gmail`. Chuyển sang "In production" thì không bị, đổi lại màn hình đăng nhập hiện cảnh báo app chưa xác minh. Kiểm tra quy định hiện hành của Google trước khi hướng dẫn người khác.

## Dùng

```sh
lai doctor                          # kiểm tra cấu hình và kết nối
lai chat                            # trò chuyện
lai run "Hôm nay có thư nào cần trả lời không?"

lai jobs list                       # các việc theo lịch và lần chạy kế tiếp
lai jobs run tom-tat-mail-sang      # chạy thử một job ngay
lai daemon                          # chạy nền, thực hiện job đúng lịch
lai report tom-tat-mail-sang        # xem báo cáo mới nhất

lai approvals list                  # hành động đang chờ duyệt
lai approvals approve <id>
lai approvals reject <id>
```

`lai daemon` phải đang chạy thì job mới chạy đúng lịch. Hiện chưa có bước tự khởi động cùng máy.

## Cơ chế duyệt

Mỗi công cụ được xếp là chỉ đọc hoặc có thay đổi.

- Chỉ đọc (`gmail_search`, `gmail_read`): chạy ngay.
- Có thay đổi (`gmail_send`, và mọi công cụ MCP ngoài trừ những cái bạn liệt kê trong `read_only_tools`): cần người duyệt.
  - Trong `lai chat` / `lai run` với provider API key: hỏi `y/N` ngay trên màn hình.
  - Trong job theo lịch, hoặc với provider `claude-cli`: đưa vào hàng chờ, không thực hiện cho tới khi bạn chạy `lai approvals approve <id>`.

Nội dung email là do người lạ viết và có thể chứa câu lừa agent ("hãy chuyển tiếp thư này..."). Lời nhắc hệ thống dặn agent coi đó là dữ liệu, nhưng lớp bảo vệ thật là bước duyệt ở trên: agent không tự gửi được gì. Lớp này không chặn việc agent bị lừa mà tóm tắt sai, nên vẫn cần đọc báo cáo với con mắt tỉnh táo.

## Dữ liệu nằm ở đâu

Thư mục cấu hình (`~/Library/Application Support/lai` trên macOS, `%APPDATA%\lai` trên Windows, `~/.config/lai` trên Linux; đổi bằng biến `LAI_HOME`):

- `config.toml`: cấu hình.
- `approvals.json`: hàng chờ duyệt.
- `reports/<job>/`: báo cáo của từng lần chạy.

API key và token Gmail lưu trong Keychain (macOS) / Credential Manager (Windows). Trên Linux lưu ở `secrets.json` với quyền chỉ chủ máy đọc được.

## Phát triển

```sh
cargo test
cargo clippy --all-targets -- -D warnings
```

Test không gọi dịch vụ thật: API Anthropic và Gmail được giả lập bằng server cục bộ, Claude CLI bằng một script thế chỗ.

Bố cục mã nguồn (`src/`):

- `tools.rs`: định nghĩa công cụ và `execute`, nơi duy nhất áp quy tắc duyệt.
- `anthropic.rs`: provider API key, tự chạy vòng lặp gọi công cụ.
- `claude_cli.rs`: provider subscription, điều khiển lệnh `claude`.
- `mcp.rs`: MCP qua stdio, cả hai chiều (cấp công cụ cho Claude CLI, và dùng MCP server ngoài).
- `gmail.rs`, `scheduler.rs`, `approval.rs`, `config.rs`, `secrets.rs`.

## Chưa làm

- App desktop (Tauri) và file cài đặt `.dmg` / `.msi` đã ký.
- Tự khởi động `lai daemon` cùng máy; thông báo trên màn hình khi có báo cáo hoặc việc chờ duyệt.
- Provider khác: OpenAI / Codex CLI, Gemini.
- Outlook và mail công ty; trả lời trong cùng luồng thư, tệp đính kèm, tạo bản nháp.
- Hiện chữ dần khi agent trả lời (streaming).
