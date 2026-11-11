---
title: "Thiết kế tool cho agent: từ tool calling đến MCP và Skills"
date: 2026-11-09
tags: [MCP, Agent, Harness]
description: Tool calling chạy thế nào ở mức API, viết tool sao cho model dùng đúng, vì sao nối nhiều MCP server làm phình context và hai lối ra hiện có. Kèm ví dụ tool definition xấu và tốt, bảng chọn giữa MCP, Skill và CLI.
loai: chuyen-sau
---

Một agent kém thường bị đổ cho model hoặc prompt. Phần ít được soi hơn là bộ tool: tên đặt thế nào, mô tả viết gì, trả về bao nhiêu, báo lỗi ra sao. Anthropic gọi tool là một loại phần mềm mới, một hợp đồng giữa hệ thống tất định (code của bạn) và một bên phi tất định (agent). Theo mình, viết tool như viết API cho lập trình viên là sai đối tượng: lập trình viên đọc tài liệu một lần rồi nhớ, còn model đọc lại toàn bộ mô tả ở mỗi lượt gọi và chỉ biết đúng những gì nằm trong context window.

Bài này đi từ dưới lên: cơ chế tool calling ở mức API, nguyên tắc viết tool, chi phí token, MCP, vấn đề phình context khi nối nhiều server, và Agent Skills. Nội dung tổng hợp từ tài liệu gốc của Anthropic, OpenAI, Cloudflare và đặc tả MCP. Câu có tên nguồn là nguồn nói. Chỗ nào là suy luận thì mình ghi "theo mình" hoặc "mình suy ra". Các ví dụ JSON về cửa hàng online là ví dụ mình tự đặt.

## Tool calling ở mức API: model không chạy gì cả

Model không thực thi tool. Nó sinh ra một khối dữ liệu có cấu trúc nói rằng "tôi muốn gọi tool này với tham số này", rồi dừng. Harness (lớp code bao quanh model, xem [Giải phẫu một agent harness](/bai-chia-se/giai-phau-agent-harness/)) mới là bên chạy thật, sau đó gửi kết quả ngược lại như một tin nhắn mới.

```text
  harness                               model
     |  tools[] + messages  ------------> |
     |  <------------  tool_use {id, name, input}   (stop_reason = tool_use)
     |  chạy code thật
     |  tool_result {tool_use_id, content} ---> |
     |  <------------  tool_use khác, hoặc câu trả lời cuối
```

Với Messages API của Anthropic, model trả về:

```json
{
  "stop_reason": "tool_use",
  "content": [
    { "type": "text", "text": "Để mình tra đơn hàng này." },
    {
      "type": "tool_use",
      "id": "toolu_01A09q90qw90lq917835lq9",
      "name": "orders_search",
      "input": { "customer_phone": "0901234567", "status": "pending" }
    }
  ]
}
```

Harness chạy hàm tương ứng rồi gửi lại một tin nhắn chứa `tool_result`, khớp với `id` ở trên:

```json
{
  "type": "tool_result",
  "tool_use_id": "toolu_01A09q90qw90lq917835lq9",
  "content": "2 đơn đang chờ: #A1042 (2 món, 450.000đ), #A1057 (1 món, 120.000đ)"
}
```

OpenAI dùng tên khác cho cùng cơ chế: model trả về mục `function_call` có `call_id` và `arguments` là một chuỗi JSON cần parse, bạn gửi lại `function_call_output` với cùng `call_id`. Cả hai bên đều có chế độ `strict` ép đầu ra khớp JSON Schema. Bên OpenAI, strict đòi mọi object có `additionalProperties: false` và mọi trường nằm trong `required`.

Cơ chế này kéo theo mấy hệ quả cho các phần sau. Tool definition là prompt. Tài liệu Anthropic viết rằng API dựng một system prompt đặc biệt từ danh sách tool. Tài liệu OpenAI viết rằng định nghĩa hàm được chèn vào system message, tính vào giới hạn context và tính tiền như input token. Bạn trả khoản này ở mọi lượt, kể cả khi không tool nào được gọi.

Kết quả tool nằm lại trong context. Một `tool_result` 20.000 token được đọc lại ở mọi lượt sau cho tới khi bị nén hoặc xóa.

Thông báo lỗi cũng là prompt, vì model đọc lỗi rồi quyết định bước tiếp theo.

## Viết tool cho agent, không phải cho lập trình viên

### Một ví dụ xấu và một ví dụ tốt

Giả sử bạn làm agent chăm sóc khách hàng cho một cửa hàng online và cần tool tra đơn. Phiên bản thường gặp khi bọc thẳng API nội bộ:

```json
{
  "name": "get_data",
  "description": "Lấy dữ liệu đơn hàng.",
  "input_schema": {
    "type": "object",
    "properties": {
      "q": { "type": "string" },
      "type": { "type": "string" },
      "from": { "type": "string" },
      "limit": { "type": "number" }
    }
  }
}
```

Model phải đoán gần như mọi thứ: `q` là mã đơn hay số điện thoại? `from` là ngày theo định dạng gì, hay là offset phân trang? Không có `limit` thì trả về bao nhiêu?

Phiên bản viết cho agent:

```json
{
  "name": "orders_search",
  "description": "Tìm đơn hàng của một khách theo số điện thoại hoặc mã đơn. Dùng khi khách hỏi về tình trạng, nội dung hoặc lịch sử đơn. Không dùng để tra tồn kho (dùng inventory_check) và không sửa được đơn. Kết quả xếp từ mới đến cũ, tối đa 10 đơn mỗi lần gọi; nếu còn nữa, kết quả có trường next_page để truyền lại vào tham số page.",
  "input_schema": {
    "type": "object",
    "properties": {
      "customer_phone": {
        "type": "string",
        "description": "Số điện thoại khách, 10 chữ số, không dấu cách, ví dụ 0901234567. Bỏ trống nếu đã có order_code."
      },
      "order_code": {
        "type": "string",
        "description": "Mã đơn dạng chữ A kèm 4 chữ số, ví dụ A1042."
      },
      "status": {
        "type": "string",
        "enum": ["pending", "shipping", "delivered", "cancelled"],
        "description": "Lọc theo trạng thái. Bỏ trống để lấy mọi trạng thái."
      },
      "created_after": {
        "type": "string",
        "description": "Chỉ lấy đơn tạo từ ngày này, định dạng YYYY-MM-DD theo giờ Việt Nam."
      },
      "response_format": {
        "type": "string",
        "enum": ["concise", "detailed"],
        "description": "concise: mã đơn, trạng thái, tổng tiền. detailed: thêm từng món, địa chỉ, mã vận đơn. Mặc định concise."
      },
      "page": { "type": "string", "description": "Giá trị next_page từ lần gọi trước." }
    }
  }
}
```

Dài hơn nhiều, và đó là chủ ý. Tài liệu tool use của Anthropic coi mô tả chi tiết là yếu tố quan trọng nhất với hiệu quả dùng tool và khuyên ít nhất 3-4 câu cho mỗi tool. Bài "Writing effective tools for agents" khuyên viết như khi giải thích cho một đồng nghiệp mới: những thứ cả đội ngầm hiểu (định dạng mã đơn, múi giờ, quan hệ giữa các tài nguyên) phải được viết ra.

### Gộp theo workflow thay vì bọc từng endpoint

Phản xạ tự nhiên là mỗi endpoint REST thành một tool. Bài "Writing effective tools for agents" phản bác: thêm tool không tự động cho kết quả tốt hơn. Một tool `list_contacts` buộc agent đọc cả danh bạ, trong khi `search_contacts` đưa nó tới đúng chỗ. Các ví dụ gộp trong bài:

- `schedule_event` thay cho chuỗi `list_users`, `list_events`, `create_event`.
- `search_logs` trả về dòng khớp kèm ngữ cảnh xung quanh, thay cho `read_logs`.
- `get_customer_context` thay cho `get_customer_by_id`, `list_transactions`, `list_notes`.

Mình hiểu là mỗi bước trung gian tốn một lượt suy luận và thêm một cơ hội chọn sai. OpenAI khuyên cùng hướng: gộp các hàm luôn được gọi nối tiếp, và đừng bắt model điền tham số mà code của bạn đã biết.

Hai nguồn của chính Anthropic lệch nhau nhẹ ở đây. Bài engineering khuyên gộp theo tác vụ của người dùng, còn tài liệu API gợi ý gộp các thao tác liên quan (`create_pr`, `review_pr`, `merge_pr`) vào một tool có tham số `action`. Theo mình, cách sau tạo ra một schema mà tham số hợp lệ phụ thuộc vào `action`, thứ JSON Schema diễn đạt kém. Mình chưa thấy số liệu công khai so sánh hai cách.

### Tên và tham số

Khi agent có hàng chục tool từ nhiều nguồn, tên là tín hiệu định tuyến đầu tiên. Anthropic khuyên đặt tiền tố theo dịch vụ và tài nguyên (`asana_projects_search`, `jira_search`), và ghi nhận việc chọn tiền tố hay hậu tố có ảnh hưởng đáng kể trên eval của họ, nên đừng chép quy ước của người khác mà không thử.

Với tham số, nguyên tắc là làm cho việc gọi sai trở nên khó (poka-yoke, khái niệm chống lỗi từ sản xuất). Ví dụ trong "Building effective agents": agent SWE-bench của Anthropic hay sai đường dẫn tương đối sau khi rời thư mục gốc. Họ đổi tool thành bắt buộc đường dẫn tuyệt đối và model dùng đúng. Họ kể đã dành nhiều thời gian tối ưu tool hơn là prompt tổng. Vài cách cụ thể:

- Tên tham số không mơ hồ: `user_id` thay vì `user` (ví dụ của Anthropic).
- Dùng `enum` cho tập giá trị đóng, đừng để chuỗi tự do rồi kiểm tra sau.
- Ghi định dạng kèm ví dụ ngay trong mô tả tham số.
- Với tham số lồng nhau hoặc nhạy định dạng, Claude API có trường `input_examples`. Thử nghiệm nội bộ của Anthropic cho thấy độ chính xác khi xử lý tham số phức tạp tăng từ 72% lên 90%. Mỗi ví dụ tốn thêm khoảng 20-200 token tùy độ phức tạp.

### Trả về ít mà đúng

Kết quả tool được đọc lại ở mọi lượt sau, nên mỗi trường thừa là khoản phí lặp lại. Các quy tắc từ bài của Anthropic:

- Trả trường mang nghĩa (`name`, `file_type`), bỏ trường kỹ thuật mà agent không dùng để suy luận (`mime_type`, URL ảnh thu nhỏ). Họ báo cáo việc đổi UUID ngẫu nhiên thành tên có nghĩa làm giảm ảo giác.
- Cho agent chọn mức chi tiết bằng `response_format`. Trong ví dụ Slack của họ, bản concise khoảng 72 token so với 206 token của bản detailed.
- Phân trang, lọc, cắt ngắn với giá trị mặc định hợp lý. Claude Code giới hạn mặc định 25.000 token cho một kết quả tool.
- Khi cắt, nói cho agent biết đã cắt và nên làm gì: lọc hẹp hơn, hay gọi trang kế.

### Thông báo lỗi dạy lại model

So sánh hai cách báo cùng một lỗi:

```text
Xấu:  Error 400: invalid input

Tốt:  created_after phải có định dạng YYYY-MM-DD, nhận được "09/11/2026".
      Ví dụ hợp lệ: 2026-11-09. Gọi lại với giá trị đã sửa.
```

Bản xấu buộc model đoán. Bản tốt nói sai ở đâu, đúng thì thế nào, bước tiếp theo là gì. Với Claude API, đặt `is_error: true` trên `tool_result`. Với MCP, lỗi của tool được trả trong kết quả với `isError: true` chứ không phải lỗi tầng giao thức, để model đọc được và tự sửa.

### Thử tool bằng tác vụ thật

Quy trình Anthropic mô tả: dựng tác vụ sát thực tế cần nhiều lượt gọi tool, mỗi tác vụ có kết quả kiểm chứng được, chạy trong một vòng lặp agent đơn giản, rồi đọc transcript. Một ví dụ trong bài: Claude tự thêm "2025" vào câu truy vấn của tool tìm kiếm web làm lệch kết quả, và sửa mô tả tool là hết. Cách dựng bộ task và grader ở bài [Eval cho ứng dụng LLM và agent](/bai-chia-se/eval-cho-agent/).

## Chi phí token của tool definitions

Vài con số để hình dung quy mô, tất cả do Anthropic công bố trong bài "Advanced tool use" (11/2025):

| Tình huống | Token cho definitions |
| --- | --- |
| Năm MCP server (GitHub, Slack, Sentry, Grafana, Splunk), tổng 58 tool | khoảng 55.000 |
| Ví dụ hơn 50 tool MCP | khoảng 72.000 |
| Trường hợp nội bộ trước khi tối ưu | 134.000 |

Ngoài tiền còn có độ chính xác. Tài liệu Anthropic viết rằng khả năng chọn đúng tool giảm khi vượt khoảng 30-50 tool. OpenAI khuyên giữ dưới 20 hàm ở đầu mỗi lượt và gọi đó là hướng dẫn mềm. Hai ngưỡng khác nhau nhưng cùng hướng: danh sách tool dài làm model chọn tệ đi trước khi làm tràn context.

Mảng `tools` còn nằm ở đầu tiền tố của prompt caching, nên thêm bớt tool giữa hội thoại làm mất cache. Tài liệu MCP cảnh báo lượt cache miss có thể tốn hơn số token bạn định tiết kiệm. Cơ chế và cách bố trí ở bài [Context engineering cho agent](/bai-chia-se/context-engineering-cho-agent/).

## MCP ở mức giao thức

Model Context Protocol (MCP) chuẩn hóa cách một ứng dụng AI lấy tool và dữ liệu từ chương trình bên ngoài. Tài liệu MCP viết rõ nó không quy định ứng dụng dùng model thế nào hay quản lý context ra sao.

```text
  +------------------- MCP host (ứng dụng AI) -------------------+
  |   model  <-->  harness                                        |
  |                   |-- MCP client 1 --stdio--> server cục bộ   |
  |                   |-- MCP client 2 --HTTP---> server từ xa    |
  +---------------------------------------------------------------+
```

Host là ứng dụng AI, client là thành phần trong host giữ kết nối tới đúng một server, server là chương trình cung cấp context.

Giao thức có hai tầng. Tầng dữ liệu là JSON-RPC 2.0 với các method như `tools/list`, `tools/call`, `resources/read`, `prompts/get`. Tầng transport có hai kiểu chuẩn:

| | stdio | Streamable HTTP |
| --- | --- | --- |
| Ai chạy server | Client khởi chạy server như tiến trình con | Server chạy độc lập, phục vụ nhiều client |
| Đóng khung | Mỗi dòng một thông điệp JSON-RPC qua stdin/stdout | Mỗi thông điệp là một HTTP POST tới một endpoint; trả về JSON hoặc luồng SSE |
| Xác thực | Dựa vào môi trường máy cục bộ | Header HTTP chuẩn, tài liệu khuyến nghị OAuth |
| Bẫy thường gặp | Server in log ra stdout làm hỏng luồng; log phải đi stderr | Phải kiểm tra header `Origin` để chặn DNS rebinding; chạy cục bộ thì chỉ bind 127.0.0.1 |

Server cung cấp ba loại primitive, theo định nghĩa của tài liệu. Tools là hàm thực thi được mà ứng dụng AI gọi để hành động. Resources là nguồn dữ liệu cung cấp ngữ cảnh (nội dung file, bản ghi cơ sở dữ liệu). Prompts là mẫu dùng lại để cấu trúc tương tác với model. Một MCP tool cũng có `name`, `description`, `inputSchema`, và host chuyển chúng thành tool definition cho model, nên mọi nguyên tắc viết tool ở phần trên áp dụng nguyên vẹn. Theo quan sát của mình (không có số liệu), phần lớn server công khai tập trung vào tools.

Bản đặc tả 2026-07-28 chuyển sang mô hình không trạng thái: bỏ bước bắt tay `initialize` và bỏ phiên ở tầng giao thức (header `Mcp-Session-Id`). Mỗi request tự mang phiên bản giao thức và capability của client trong `_meta`. Server bắt buộc có method `server/discover` để client hỏi phiên bản và khả năng, và client được phép gọi thẳng mà không hỏi trước. Thông báo thay đổi (ví dụ danh sách tool đổi) đi qua một luồng `subscriptions/listen` mà client chủ động mở. Bản này cũng đưa Roots, Sampling và Logging vào diện deprecated. Transport HTTP+SSE của bản 2024-11-05 vốn đã deprecated từ trước. Hướng dẫn cũ khác với SDK mới là vì vậy.

Theo mình, điều MCP không giải quyết là chất lượng tool. Một server bọc 80 endpoint thành 80 tool vẫn là 80 tool viết cho lập trình viên, chỉ khác là giờ ai cũng cài được nhanh. Rủi ro bảo mật riêng của MCP (mô tả tool độc hại, server bị đổi nội dung) ở bài [Bảo mật cho agent](/bai-chia-se/bao-mat-agent-prompt-injection/).

## Khi nối nhiều server: phình context và các lối ra

Cách triển khai ngây thơ là host gọi `tools/list` trên mọi server rồi nhét toàn bộ định nghĩa vào prompt từ lượt đầu. Tài liệu MCP viết rằng với hàng chục server, riêng định nghĩa có thể chiếm phần lớn context trước khi model đọc câu hỏi. Vấn đề thứ hai là kết quả trung gian. Ví dụ của Anthropic: chuyển một biên bản họp từ Google Drive sang Salesforce bằng tool call trực tiếp thì nội dung biên bản đi qua model hai lần, một lần khi đọc và một lần khi model chép nó vào tham số của lệnh ghi.

### Lối ra 1: tool search và nạp trễ

Model chỉ thấy một tool tìm kiếm và vài tool dùng thường xuyên. Phần còn lại được đánh dấu `defer_loading: true` và chỉ vào context khi model tìm ra. Anthropic có sẵn hai biến thể (regex và BM25), OpenAI có `tool_search` cùng khái niệm namespace để nhóm tool theo miền, và trang hướng dẫn cho client của MCP mô tả cùng pattern ở phía host dưới tên progressive discovery, gợi ý đặt ngưỡng chuyển sang chế độ này theo phần trăm context window, ví dụ 1-5%.

Theo số liệu Anthropic công bố, trong ví dụ của họ, context dùng trước khi bắt đầu việc giảm từ khoảng 77.000 xuống 8.700 token. Trên bộ eval MCP nội bộ, độ chính xác của Opus 4 tăng từ 49% lên 74%, của Opus 4.5 từ 79,5% lên 88,1%. Đây là số nội bộ, không kèm phương pháp đầy đủ, nên coi là chỉ dấu về hướng chứ không phải mức bạn sẽ đạt.

Cách này có giới hạn. Theo tài liệu tool search của Claude, bạn vẫn gửi toàn bộ định nghĩa trong mỗi request (chúng chỉ không vào context), và mỗi lần tìm mặc định trả về tối đa 5 tool. Tìm kiếm chạy trên tên, mô tả, tên tham số và mô tả tham số, nên tiền tố nhất quán và từ khóa trong mô tả là điều kiện để tool được tìm thấy. Tài liệu khuyên không dùng khi có dưới 10 tool hoặc mọi tool đều dùng ở mọi request. Mình suy ra thêm một cái giá: mỗi lần tìm là một bước nữa trước khi gọi được tool, tức thêm độ trễ.

### Lối ra 2: cho agent viết code gọi tool

Thay vì gọi từng tool rồi nhận từng kết quả, host biến các MCP tool thành một API có kiểu (ví dụ một cây file TypeScript, mỗi tool một file) và cho model một tool duy nhất để chạy code trong sandbox. Model đọc đúng file nó cần, viết một đoạn script gọi nhiều tool, lọc và ghép dữ liệu ngay trong sandbox, rồi chỉ in ra phần tóm tắt. Đoạn dưới rút gọn từ ví dụ trong tài liệu MCP:

```ts
// Code do model viết, chạy trong sandbox
const logs = await logging_getLogs({ level: "error", since: Date.now() - 3600000 });

const unique = new Set<string>();
for (const log of logs.entries) unique.add(log.message);

for (const message of unique) {
  await ticketing_createIssue({ title: `Error: ${message}`, priority: "high" });
}
console.log(`Đã tạo ${unique.size} ticket từ ${logs.entries.length} dòng log`);
```

Hàng nghìn dòng log không vào context. Model chỉ thấy một dòng tổng kết, và vòng lặp chạy bằng code thay vì mỗi bước một lượt suy luận. Cloudflare, bên gọi cách này là Code Mode, lập luận rằng model đã thấy nhiều code trong dữ liệu huấn luyện nhưng ít thấy tool call, nên viết code gọi API là việc nó làm tốt hơn. Anthropic đưa con số giảm từ 150.000 xuống 2.000 token (98,7%) trong bài "Code execution with MCP". Bài không mô tả cách đo, nên mình coi đó là kịch bản minh họa. Con số có nêu bối cảnh đo thì khiêm tốn hơn: tính năng programmatic tool calling giảm trung bình 37% token trên các tác vụ nghiên cứu phức tạp.

Cái giá là một bề mặt tấn công mới. Tài liệu MCP liệt kê: sandbox không có mạng trực tiếp, giới hạn tài nguyên, host làm broker giữ thông tin xác thực và kiểm tra từng lệnh gọi. Duyệt một script không có nghĩa là duyệt mọi lệnh nó gọi lúc chạy. Kết quả từ server này là đầu vào không đáng tin với server kia. Theo mình, nếu bạn chưa có hạ tầng sandbox thì lối ra 1 rẻ hơn nhiều.

## Agent Skills và progressive disclosure

Tool trả lời câu "agent làm được gì", còn skill trả lời câu "agent làm việc này theo cách của đội bạn thế nào". Một skill là một thư mục có file `SKILL.md` (frontmatter YAML với `name` và `description`, bên dưới là hướng dẫn) cùng script và tài liệu tham chiếu tùy chọn.

```text
bao-cao-doanh-thu/
├── SKILL.md            # name, description + quy trình
├── references/
│   └── schema.md       # chỉ đọc khi cần viết truy vấn
└── scripts/
    └── tong_hop.py     # chạy, không cần đọc mã
```

Skill được nạp theo ba tầng, gọi là progressive disclosure (tiết lộ dần). Số liệu theo tài liệu Agent Skills của Anthropic và đặc tả agentskills.io:

| Tầng | Nạp khi nào | Chi phí |
| --- | --- | --- |
| Metadata (`name`, `description`) | Luôn có trong system prompt | Khoảng 100 token mỗi skill |
| Thân `SKILL.md` | Khi agent thấy tác vụ khớp mô tả | Khuyến nghị dưới 5.000 token, dưới 500 dòng |
| File tham chiếu, script | Khi hướng dẫn trỏ tới | Không tốn gì cho tới khi đọc; script chỉ tốn phần output |

Đây là cùng ý tưởng nạp trễ như tool search, làm ở tầng hệ thống file. Điều kiện đi kèm là agent phải có hệ thống file và quyền chạy code.

`description` là thứ agent đối chiếu để quyết định có kích hoạt skill không, nên phải nói cả skill làm gì lẫn khi nào dùng (giới hạn 1.024 ký tự). Mình suy ra mô tả kiểu "Hỗ trợ xử lý PDF" sẽ khiến skill ít được gọi đúng lúc.

Việc cần kết quả lặp lại chính xác (parse file, tính toán, kiểm tra định dạng) thì đưa vào script, vì code cho kết quả giống nhau mỗi lần và mã nguồn của script không vào context.

Về an toàn, tài liệu khuyên coi việc cài skill như cài phần mềm: skill là hướng dẫn cộng code chạy với quyền của agent, nên một skill độc hại có thể bảo agent gửi dữ liệu ra ngoài. Đọc hết các file, nhất là script và mọi chỉ dẫn gọi tới địa chỉ bên ngoài, trước khi dùng skill không phải do bạn viết.

## Chọn gì: MCP, Skill hay CLI thuần

Còn một lựa chọn hay bị quên là không thêm gì cả, để agent dùng công cụ dòng lệnh sẵn có qua tool chạy shell. Cả mục này là suy luận của mình, mở rộng từ lập luận của Cloudflare về code: model đã thấy `git`, `gh`, `curl`, `jq` nhiều lần, các lệnh đó có `--help` để tự tra, và chúng không tốn token định nghĩa. Bảng dưới chưa được kiểm bằng thí nghiệm nào.

| Tiêu chí | CLI thuần | MCP server | Skill |
| --- | --- | --- | --- |
| Bản chất | Lệnh có sẵn, gọi qua shell | Tool có schema, qua giao thức | Hướng dẫn + script trên hệ thống file |
| Chi phí context thường trực | Gần 0 | Toàn bộ định nghĩa, trừ khi nạp trễ | Khoảng 100 token mỗi skill |
| Cần ở phía agent | Shell, tốt nhất có sandbox | Host hỗ trợ MCP | Hệ thống file và chạy code |
| Xác thực và phân quyền | Theo máy đang chạy, khó giới hạn từng thao tác | Theo từng server, có OAuth cho server từ xa | Thừa hưởng quyền của agent |
| Hợp nhất khi | Công cụ phổ biến, agent chạy trên máy lập trình viên | Dịch vụ từ xa, agent không có shell, cần kiểm soát từng thao tác | Quy trình và tri thức riêng của đội |
| Điểm yếu | Output viết cho người, dễ dài; lệnh phá hoại khó chặn | Phình context, chất lượng tùy người viết server | Phụ thuộc mô tả để kích hoạt; rủi ro từ skill lạ |

Ba thứ này không loại trừ nhau: MCP cho kết nối tới hệ thống cần xác thực, skill để dạy quy trình dùng các tool đó, CLI cho những thứ model vốn đã thạo.

## Checklist trước khi phát hành một tool

- Tên có tiền tố dịch vụ, không trùng nghĩa với tool khác.
- Mô tả trả lời đủ: làm gì, khi nào dùng, khi nào không, trả về gì, giới hạn gì.
- Mỗi tham số có mô tả, định dạng và ví dụ. Tập giá trị đóng dùng `enum`.
- Có giá trị mặc định cho phân trang và mức chi tiết. Kết quả bị cắt thì nói rõ cách lấy tiếp.
- Kết quả không chứa trường mà agent không dùng để quyết định bước sau.
- Mọi lỗi có ba phần: sai gì, đúng thì thế nào, làm gì tiếp.
- Thao tác ghi hoặc xóa được đánh dấu rõ và có bước xác nhận ở harness.
- Đã chạy ít nhất một bộ tác vụ thực tế và đọc transcript, ngoài việc nhìn tỉ lệ đạt.
- Đã đếm token của toàn bộ định nghĩa và quyết định tool nào nạp sẵn, tool nào nạp trễ.

## Điều còn mở / chưa chắc

- Gọi tool trực tiếp hay viết code gọi tool sẽ thành mặc định? Anthropic và Cloudflare đều đẩy hướng code, nhưng nó đòi sandbox mà nhiều đội chưa có. Mình chưa thấy benchmark độc lập so sánh hai cách trên cùng bộ tác vụ.
- Skill có thay MCP không? Hai thứ giải hai việc khác nhau, nhưng ranh giới đang mờ đi khi skill có thể bọc script gọi API. Với dịch vụ cần xác thực theo từng người dùng, mình nghĩ MCP vẫn cần, song đây là nhận định chứ chưa phải điều đã được chứng minh.
- Đặc tả MCP còn đổi nhanh. Bản 2026-07-28 thay cả mô hình kết nối, và mình chưa kiểm được các SDK và client phổ biến đã theo bản này tới đâu. Chi tiết giao thức trong bài có thể cũ đi sau vài tháng.
- Phần lớn số liệu trong bài do chính các nhà cung cấp công bố, đo trên model và bộ eval của họ.

## Nguồn đọc thêm

- [Writing effective tools for agents](https://www.anthropic.com/engineering/writing-tools-for-agents) (Anthropic): nguồn chính cho phần nguyên tắc viết tool, có quy trình eval cụ thể.
- [Define tools](https://platform.claude.com/docs/en/agents-and-tools/tool-use/define-tools) và [Handle tool calls](https://platform.claude.com/docs/en/agents-and-tools/tool-use/handle-tool-calls) (tài liệu Claude API): cấu trúc `tool_use`, `tool_result`, ví dụ mô tả tốt và xấu.
- [Function calling](https://developers.openai.com/api/docs/guides/function-calling) (tài liệu OpenAI): cùng cơ chế với tên gọi khác, phần strict mode và namespace đáng đọc.
- [Advanced tool use](https://www.anthropic.com/engineering/advanced-tool-use) (Anthropic, 11/2025) và [Tool search tool](https://platform.claude.com/docs/en/agents-and-tools/tool-use/tool-search-tool) (tài liệu Claude API): số liệu về nạp trễ, programmatic tool calling và `input_examples`.
- [Code execution with MCP](https://www.anthropic.com/engineering/code-execution-with-mcp) (Anthropic, 11/2025) và [Code Mode](https://blog.cloudflare.com/code-mode/) (Cloudflare, 9/2025): hai bài cùng đi tới ý tưởng cho agent viết code gọi tool.
- [MCP: Architecture overview](https://modelcontextprotocol.io/docs/learn/architecture), [changelog bản 2026-07-28](https://modelcontextprotocol.io/specification/2026-07-28/changelog), đặc tả [stdio](https://modelcontextprotocol.io/specification/2026-07-28/basic/transports/stdio) và [Streamable HTTP](https://modelcontextprotocol.io/specification/2026-07-28/basic/transports/streamable-http): đọc để hiểu giao thức thật thay vì qua lớp SDK.
- [MCP: Client best practices](https://modelcontextprotocol.io/docs/2026-07-28/develop/clients/client-best-practices): progressive discovery và code mode nhìn từ phía người viết host, có phần bảo mật kỹ.
- [Equipping agents for the real world with Agent Skills](https://www.anthropic.com/engineering/equipping-agents-for-the-real-world-with-agent-skills) (Anthropic), [Agent Skills overview](https://platform.claude.com/docs/en/agents-and-tools/agent-skills/overview) và [đặc tả Agent Skills](https://agentskills.io/specification): ba tầng nạp và định dạng `SKILL.md`.
- [Building effective agents](https://www.anthropic.com/engineering/building-effective-agents) (Anthropic, 12/2024): phụ lục về agent-computer interface và ví dụ đường dẫn tuyệt đối.
