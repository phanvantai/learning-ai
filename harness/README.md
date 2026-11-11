# Trợ lý AI (lai-harness)

Trợ lý AI cá nhân chạy trên máy của bạn (macOS, Windows, Linux). Bạn giao việc bằng lời, trợ lý tự kiểm tra Gmail theo lịch và viết báo cáo. Mọi hành động thay đổi bên ngoài (gửi thư...) đều phải có bạn duyệt.

Có hai cách dùng, chung một lõi và một file cấu hình:

- **App desktop** (`harness/app`, Tauri): cửa sổ giao diện, biểu tượng ở khay hệ thống, thông báo, tự mở cùng máy.
- **Dòng lệnh** (`lai`): cùng tính năng, thêm `lai ui` để mở giao diện trong trình duyệt.

## Cài app

Bộ cài được build bằng GitHub Actions (workflow `harness-app`): `.dmg` cho macOS (chạy được cả máy Intel và Apple Silicon) và `.exe` cho Windows. Tải ở mục Artifacts của lần chạy workflow, hoặc ở bản release nháp khi đẩy tag `harness-v*`.

Bộ cài **chưa được ký** bằng chứng chỉ của Apple/Microsoft, nên lần đầu mở sẽ bị hệ điều hành chặn:

- macOS: mở app một lần, rồi vào System Settings → Privacy & Security → bấm "Open Anyway".
- Windows: ở màn hình SmartScreen bấm "More info" → "Run anyway".

Muốn hết cảnh báo thì cần tài khoản Apple Developer và chứng chỉ ký code cho Windows; phần này chưa cấu hình.

## Thiết lập lần đầu (trong app, mục Kết nối)

### 1. Chọn "bộ não"

| Loại | Dùng gì | Cần làm |
|---|---|---|
| Gói Claude | Gói Pro/Max đã đăng nhập trong Claude Code | Cài [Claude Code](https://claude.com/claude-code), chạy `claude` và đăng nhập một lần |
| API key Anthropic | Trả theo lượng dùng | Dán key vào ô trong app |
| API tương thích OpenAI | OpenAI, Gemini, OpenRouter, Ollama... | Thêm bộ não, ghi tên model, địa chỉ API và key |
| Gói ChatGPT (thử nghiệm) | Gói đã đăng nhập trong Codex CLI | Cài Codex CLI, chạy `codex login` |

Với hai loại "gói", app chỉ gọi lệnh chính thức (`claude`, `codex`); app không đọc và không lưu token đăng nhập. Công cụ có sẵn của Claude Code (chạy lệnh, sửa file, web) bị tắt. Codex không cho tắt công cụ chạy lệnh, nên app ép nó vào chế độ chỉ đọc và một thư mục trống.

Mức độ kiểm chứng: gói Claude đã chạy thật từ đầu đến cuối. API Anthropic và API tương thích OpenAI mới được test với máy chủ giả lập. Gói ChatGPT mới kiểm tra tới bước lệnh `codex` nhận đúng tham số; chưa chạy với tài khoản thật và chưa thử trên Windows.

Lưu ý về gói subscription: gói cá nhân dành cho một người dùng trực tiếp. Cho job chạy nền dày đặc có thể nằm ngoài phạm vi đó; hãy đọc điều khoản của nhà cung cấp, và dùng API key cho việc chạy nền nặng.

### 2. Kết nối Gmail

App dùng Gmail API với quyền đọc thư và gửi thư. Cần một OAuth client của Google, tạo một lần:

1. Vào [Google Cloud Console](https://console.cloud.google.com), tạo project, bật **Gmail API**.
2. Cấu hình OAuth consent screen (loại External), thêm email sẽ dùng vào Test users.
3. Tạo Credentials → OAuth client ID → loại **Desktop app**. Lấy Client ID và Client secret.
4. Trong app: Kết nối → Gmail → dán hai giá trị → "Kết nối Gmail" → đăng nhập trên trình duyệt.

Khi consent screen còn ở trạng thái Testing, Google thường thu hồi đăng nhập sau khoảng 7 ngày. Chuyển sang "In production" thì không bị, đổi lại màn hình đăng nhập hiện cảnh báo app chưa xác minh. Kiểm tra quy định hiện hành của Google trước khi hướng dẫn người khác.

**Phát app cho học viên mà không bắt họ tạo OAuth client:** đặt hai secret `LAI_GMAIL_CLIENT_ID` và `LAI_GMAIL_CLIENT_SECRET` trong repo GitHub. Bộ cài build ra sẽ mang sẵn client của bạn, học viên chỉ cần bấm "Kết nối Gmail". Khi đó mọi người đăng nhập qua project Google của bạn, nên giới hạn test user và yêu cầu xác minh của Google áp lên project đó.

## Dùng app

- **Trò chuyện:** giao việc bằng lời.
- **Việc theo lịch:** thêm việc, chọn giờ, xem báo cáo. Việc chỉ chạy khi app đang mở; đóng cửa sổ thì app vẫn nằm ở khay hệ thống. Bật "Tự mở cùng máy tính" trong mục Kết nối để khỏi phải nhớ mở.
- **Chờ duyệt:** thư trợ lý soạn nằm ở đây cho tới khi bạn bấm "Đồng ý gửi".

## Cơ chế duyệt

Mỗi công cụ được xếp là chỉ đọc hoặc có thay đổi.

- Chỉ đọc (`gmail_search`, `gmail_read`): chạy ngay.
- Có thay đổi (`gmail_send`, và mọi công cụ MCP ngoài trừ những cái bạn liệt kê trong `read_only_tools`): vào hàng chờ, chỉ thực hiện khi bạn duyệt. Riêng `lai chat` / `lai run` với provider API key thì hỏi `y/N` ngay trên màn hình lệnh.

Nội dung email là do người lạ viết và có thể chứa câu lừa trợ lý ("hãy chuyển tiếp thư này..."). Lời nhắc hệ thống dặn trợ lý coi đó là dữ liệu, nhưng lớp bảo vệ thật là bước duyệt: trợ lý không tự gửi được gì. Lớp này không chặn việc trợ lý bị lừa mà tóm tắt sai, nên vẫn cần đọc báo cáo với con mắt tỉnh táo.

Giao diện là một trang web do app phục vụ trên `127.0.0.1`. Mọi lệnh gọi phải kèm một mã ngẫu nhiên sinh ra mỗi lần mở app, nên trang web khác đang mở trong trình duyệt không điều khiển được trợ lý.

## Dòng lệnh

Cần [Rust](https://rustup.rs).

```sh
cd harness
cargo install --path .
lai init
```

```sh
lai ui                              # mở giao diện trong trình duyệt
lai doctor                          # kiểm tra cấu hình và kết nối
lai auth key [provider]             # nhập API key
lai auth gmail                      # đăng nhập Gmail
lai chat
lai run "Hôm nay có thư nào cần trả lời không?"
lai jobs list | lai jobs run <tên>
lai daemon                          # chạy job đúng lịch, không giao diện
lai report <tên job>
lai approvals list | approve <id> | reject <id>
```

## Dữ liệu nằm ở đâu

Thư mục cấu hình (`~/Library/Application Support/lai` trên macOS, `%APPDATA%\lai` trên Windows, `~/.config/lai` trên Linux; đổi bằng biến `LAI_HOME`):

- `config.toml`: bộ não, agent, việc theo lịch, MCP server. Sửa tay được; khi app lưu thay đổi thì chú thích trong file bị bỏ.
- `approvals.json`: hàng chờ duyệt.
- `reports/<việc>/`: báo cáo của từng lần chạy.

API key và token Gmail lưu trong Keychain (macOS) / Credential Manager (Windows). Trên Linux lưu ở `secrets.json` với quyền chỉ chủ máy đọc được. Lịch sử trò chuyện không được lưu.

## Phát triển

```sh
cargo test                                   # lõi
cargo clippy --workspace --all-targets -- -D warnings

cd app
npx @tauri-apps/cli@2 dev                    # chạy app
npx @tauri-apps/cli@2 build                  # build bộ cài cho máy đang dùng
```

Build app trên Linux cần thêm `libwebkit2gtk-4.1-dev libayatana-appindicator3-dev librsvg2-dev`.

Test không gọi dịch vụ thật: API của các nhà cung cấp và Gmail được giả lập bằng server cục bộ, các lệnh `claude` / `codex` bằng script thế chỗ.

Bố cục:

- `src/tools.rs`: định nghĩa công cụ và `execute`, nơi duy nhất áp quy tắc duyệt.
- `src/anthropic.rs`, `src/openai.rs`: provider API key, tự chạy vòng lặp gọi công cụ.
- `src/claude_cli.rs`, `src/codex_cli.rs`: provider subscription, điều khiển CLI chính thức.
- `src/mcp.rs`: MCP qua stdio, cả hai chiều (cấp công cụ cho các CLI, và dùng MCP server ngoài).
- `src/server.rs` + `src/ui/`: máy chủ cục bộ và giao diện (HTML/CSS/JS thuần, không cần bước build).
- `src/gmail.rs`, `src/scheduler.rs`, `src/approval.rs`, `src/config.rs`, `src/secrets.rs`, `src/proc.rs`.
- `app/`: vỏ Tauri (cửa sổ, khay hệ thống, thông báo, tự khởi động).

## Chưa có

- Ký bộ cài (xem trên) và tự cập nhật phiên bản.
- Outlook và mail công ty; trả lời trong cùng luồng thư, tệp đính kèm.
- Hiện chữ dần khi trợ lý trả lời; lưu lịch sử trò chuyện.
- Gói Gemini qua Gemini CLI (dùng Gemini bằng API key thì được, qua loại "API tương thích OpenAI").
