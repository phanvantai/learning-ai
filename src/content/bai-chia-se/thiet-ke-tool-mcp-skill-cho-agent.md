---
title: "Thiết kế tool cho agent: từ tool calling đến MCP và Skills"
date: 2026-11-09
tags: [MCP, Agent, Harness]
description: Tool calling chạy thế nào ở mức API, viết tool sao cho model dùng đúng, vì sao nối nhiều MCP server làm phình context và ba lối ra hiện có. Kèm ví dụ tool definition xấu và tốt, bảng chọn giữa MCP, Skill và CLI.
loai: chuyen-sau
---

Một agent kém thường bị đổ cho model hoặc prompt. Phần ít được soi hơn là bộ tool: tên đặt thế nào, mô tả viết gì, trả về bao nhiêu, báo lỗi ra sao. Anthropic gọi tool là một loại phần mềm mới, một hợp đồng giữa hệ thống xác định (code của bạn) và một bên không xác định (model). Viết tool như viết API cho lập trình viên là sai đối tượng: lập trình viên đọc tài liệu một lần rồi nhớ, còn model đọc lại toàn bộ mô tả ở mỗi lượt gọi và chỉ biết đúng những gì nằm trong context window.

Bài này đi từ dưới lên: cơ chế tool calling ở mức API, nguyên tắc viết tool, chi phí token, MCP, vấn đề phình context khi nối nhiều server, và Agent Skills. Nội dung tổng hợp từ tài liệu gốc của Anthropic, OpenAI, Cloudflare và đặc tả MCP; chỗ nào là suy luận của mình thì mình nói rõ.

## Tool calling ở mức API: model không chạy gì cả

Model không thực thi tool. Nó chỉ sinh ra một khối dữ liệu có cấu trúc nói rằng "tôi muốn gọi tool này với tham số này", rồi dừng. Phần harness (đoạn code bao quanh model, lo vòng lặp và thực thi) mới là bên chạy thật, sau đó gửi kết quả ngược lại như một tin nhắn mới.

```text
  harness                               model
     |  tools[] + messages  ------------> |
     |  <------------  tool_use {id, name, input}   (stop_reason = tool_use)
     |  chạy code thật
     |  tool_result {tool_use_id, content} ---> |
     |  <------------  tool_use khác, hoặc câu trả lời cuối
```

Với Messages API của Anthropic, một lượt gọi tool trông như sau. Model trả về:

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

OpenAI dùng tên khác cho cùng cơ chế: tool khai báo với `type: "function"`, model trả về mục `function_call` có `call_id` và `arguments` là một chuỗi JSON cần parse, bạn gửi lại `function_call_output` với cùng `call_id`. Cả hai bên đều có chế độ `strict` ép đầu ra khớp JSON Schema. Bên OpenAI, strict đòi mọi object có `additionalProperties: false` và mọi trường nằm trong `required` (trường tùy chọn thì thêm `null` vào kiểu).

Ba hệ quả của cơ chế này chi phối mọi thứ phía sau:

1. Tool definition là prompt. Tài liệu Anthropic nói API dựng một system prompt đặc biệt từ danh sách tool; tài liệu OpenAI nói định nghĩa hàm được chèn vào system message, tính vào giới hạn context và tính tiền như input token. Bạn trả khoản này ở mọi lượt, kể cả khi không tool nào được gọi.
2. Kết quả tool nằm lại trong context. Một `tool_result` 20.000 token sẽ được đọc lại ở mọi lượt sau cho tới khi bị nén hoặc xóa.
3. Thông báo lỗi cũng là prompt. Model đọc lỗi rồi quyết định bước tiếp theo, nên lỗi viết tệ dẫn tới hành vi tệ.

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

Model phải đoán gần như mọi thứ: `q` là mã đơn, tên khách hay số điện thoại? `type` nhận giá trị nào? `from` là ngày theo định dạng gì, hay là offset phân trang? Không có `limit` thì trả về bao nhiêu? Tên `get_data` cũng sẽ đụng với mọi tool "lấy dữ liệu" khác.

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

Dài hơn nhiều, và đó là chủ ý. Tài liệu tool use của Anthropic coi mô tả chi tiết là yếu tố quan trọng nhất với hiệu quả dùng tool, khuyên ít nhất 3-4 câu cho mỗi tool. Những thứ "ai trong team cũng biết" (định dạng mã đơn, múi giờ, quan hệ giữa các bảng) phải được viết ra, như khi giải thích cho một đồng nghiệp mới vào.

### Gộp theo workflow thay vì bọc từng endpoint

Phản xạ tự nhiên là mỗi endpoint REST thành một tool. Bài "Writing effective tools for agents" phản bác thẳng: thêm tool không tự động cho kết quả tốt hơn. Một tool `list_contacts` buộc agent đọc cả danh bạ từng token một, trong khi `search_contacts` đưa nó tới đúng chỗ. Các ví dụ gộp trong bài:

- `schedule_event` thay cho chuỗi `list_users`, `list_events`, `create_event`.
- `search_logs` trả về dòng khớp kèm ngữ cảnh xung quanh, thay cho `read_logs`.
- `get_customer_context` thay cho `get_customer_by_id`, `list_transactions`, `list_notes`.

Cơ chế đằng sau: mỗi bước trung gian là một lượt suy luận tốn token và là một cơ hội chọn sai. OpenAI nói cùng ý: gộp các hàm luôn được gọi nối tiếp, và đừng bắt model điền tham số mà code của bạn đã biết.

Có một điểm hai nguồn của chính Anthropic lệch nhau nhẹ. Bài engineering khuyên gộp theo tác vụ của người dùng, còn tài liệu API gợi ý gộp các thao tác liên quan (`create_pr`, `review_pr`, `merge_pr`) vào một tool có tham số `action`. Cách sau giảm số tool nhưng tạo ra một schema mà tham số hợp lệ phụ thuộc vào `action`, thứ JSON Schema diễn đạt kém. Mình chưa thấy số liệu công khai so sánh hai cách, nên nếu phân vân thì đo trên tác vụ của bạn.

### Tên, namespacing và tham số

Khi agent có hàng chục tool từ nhiều nguồn, tên là tín hiệu định tuyến đầu tiên. Đặt tiền tố theo dịch vụ và tài nguyên (`asana_projects_search`, `jira_search`) để model phân biệt ranh giới. Anthropic ghi nhận việc chọn tiền tố hay hậu tố có ảnh hưởng đo được và khác nhau giữa các model, nên đừng chép quy ước của người khác mà không thử.

Với tham số, nguyên tắc là làm cho việc gọi sai trở nên khó (poka-yoke, khái niệm chống lỗi từ sản xuất). Ví dụ kinh điển trong "Building effective agents": agent SWE-bench của Anthropic hay sai đường dẫn tương đối sau khi đổi thư mục; họ đổi tool thành bắt buộc đường dẫn tuyệt đối và lỗi biến mất. Vài cách cụ thể:

- Tên tham số không mơ hồ: `user_id` thay vì `user`.
- Dùng `enum` cho tập giá trị đóng, đừng để chuỗi tự do rồi kiểm tra sau.
- Ghi định dạng kèm ví dụ ngay trong mô tả tham số.
- Với tham số lồng nhau hoặc nhạy định dạng, Anthropic có trường `input_examples`. Thử nghiệm nội bộ của họ cho thấy độ chính xác khi xử lý tham số phức tạp tăng từ 72% lên 90%, đổi lại mỗi ví dụ tốn thêm khoảng 20-200 token.

### Trả về ít mà đúng

Kết quả tool là thứ model phải đọc lại ở mọi lượt sau, nên mỗi trường thừa là thuế vĩnh viễn. Các quy tắc có nguồn:

- Trả trường mang nghĩa (`name`, `file_type`), bỏ trường kỹ thuật mà agent không dùng để suy luận (`mime_type`, URL ảnh thu nhỏ). Anthropic báo cáo việc đổi UUID ngẫu nhiên thành tên có nghĩa hoặc chỉ số đơn giản làm giảm ảo giác khi model phải chép lại định danh.
- Cho agent chọn mức chi tiết bằng `response_format`. Trong ví dụ Slack của Anthropic, bản concise khoảng 72 token so với 206 token của bản detailed, tức khoảng một phần ba.
- Phân trang, lọc, cắt ngắn với giá trị mặc định hợp lý. Claude Code giới hạn mặc định 25.000 token cho một kết quả tool.
- Khi cắt, nói cho agent biết đã cắt và nên làm gì: lọc hẹp hơn, hay gọi trang kế.

### Thông báo lỗi dạy lại model

So sánh hai cách báo cùng một lỗi:

```text
Xấu:  Error 400: invalid input

Tốt:  created_after phải có định dạng YYYY-MM-DD, nhận được "09/11/2026".
      Ví dụ hợp lệ: 2026-11-09. Gọi lại với giá trị đã sửa.
```

Bản xấu buộc model đoán, và nó thường đoán bằng cách thử lại y hệt hoặc bỏ cuộc. Bản tốt là một bài học một lần: sai ở đâu, đúng thì thế nào, bước tiếp theo là gì. Với Anthropic, đặt `is_error: true` trên `tool_result`; với MCP, lỗi của tool được trả trong kết quả với `isError: true` chứ không phải lỗi tầng giao thức, để model đọc được và tự sửa.

### Đo, đừng đoán

Các nguyên tắc trên là giả thuyết cho tới khi bạn có eval. Quy trình Anthropic mô tả: dựng tác vụ sát thực tế cần nhiều lượt gọi tool, mỗi tác vụ có kết quả kiểm chứng được, chạy trong một vòng lặp agent đơn giản, rồi đọc transcript thô. Nhiều lượt gọi lặp gợi ý phân trang hoặc giới hạn token chưa hợp lý; nhiều lỗi tham số gợi ý mô tả chưa rõ. Một ví dụ trong bài: Claude tự thêm "2025" vào câu truy vấn của tool tìm kiếm web làm lệch kết quả, và chỉ cần sửa mô tả tool là hết.

## Chi phí token của tool definitions

Vài con số để hình dung quy mô, tất cả do Anthropic công bố từ thử nghiệm của họ:

| Tình huống | Token cho definitions |
| --- | --- |
| Năm MCP server phổ biến (GitHub, Slack, Sentry, Grafana, Splunk) | khoảng 55.000 |
| Ví dụ hơn 50 tool MCP trong bài "Advanced tool use" | khoảng 72.000 |
| Trường hợp nội bộ trước khi tối ưu | 134.000 |

Chi phí không chỉ là tiền. Tài liệu Anthropic nói độ chính xác chọn tool giảm khi vượt khoảng 30-50 tool; OpenAI khuyên giữ dưới 20 hàm ở đầu mỗi lượt. Hai ngưỡng khác nhau, đo trên model khác nhau, nhưng cùng hướng: danh sách tool dài làm model chọn tệ đi trước khi làm tràn context.

Điểm thứ ba ít người để ý là prompt caching (cơ chế nhà cung cấp lưu phần đầu prompt đã xử lý để tính rẻ hơn ở lượt sau). Mảng `tools` nằm trong phần đầu đó. Thêm bớt tool giữa hội thoại làm mất cache, và theo tài liệu MCP, lượt cache miss có thể tốn hơn cả số token bạn định tiết kiệm. Hệ quả thiết kế: giữ danh sách tool ổn định trong một hội thoại, hoặc dùng cơ chế nạp trễ nói ở dưới.

## MCP ở mức giao thức

Model Context Protocol (MCP) chuẩn hóa cách một ứng dụng AI lấy tool và dữ liệu từ chương trình bên ngoài. Nó không quy định ứng dụng dùng model thế nào hay quản lý context ra sao; nó chỉ là giao thức trao đổi.

```text
  +------------------- MCP host (ứng dụng AI) -------------------+
  |   model  <-->  harness                                        |
  |                   |-- MCP client 1 --stdio--> server cục bộ   |
  |                   |-- MCP client 2 --HTTP---> server từ xa    |
  +---------------------------------------------------------------+
```

Ba vai: host là ứng dụng AI, client là thành phần trong host giữ kết nối tới đúng một server, server là chương trình cung cấp context. Host tạo một client cho mỗi server.

Giao thức có hai tầng. Tầng dữ liệu là JSON-RPC 2.0 với các method như `tools/list`, `tools/call`, `resources/read`, `prompts/get`. Tầng transport có hai kiểu chuẩn:

| | stdio | Streamable HTTP |
| --- | --- | --- |
| Ai chạy server | Client khởi chạy server như tiến trình con | Server chạy độc lập, phục vụ nhiều client |
| Đóng khung | Mỗi dòng một thông điệp JSON-RPC qua stdin/stdout | Mỗi thông điệp là một HTTP POST tới một endpoint; trả về JSON hoặc luồng SSE |
| Xác thực | Dựa vào môi trường máy cục bộ | Header HTTP chuẩn, đặc tả khuyến nghị OAuth |
| Bẫy thường gặp | Server in log ra stdout làm hỏng luồng; log phải đi stderr | Phải kiểm tra header `Origin` để chặn DNS rebinding; chạy cục bộ thì chỉ bind 127.0.0.1 |

Server cung cấp ba loại primitive. Tools là hàm thực thi được, do model quyết định gọi. Resources là dữ liệu chỉ đọc (nội dung file, schema cơ sở dữ liệu). Prompts là mẫu tương tác dùng lại. Trên thực tế đa số server chỉ triển khai tools, và mọi nguyên tắc viết tool ở phần trên áp dụng nguyên vẹn: một MCP tool cũng có `name`, `description`, `inputSchema`, và host sẽ chuyển chúng thành tool definition cho model.

Một lưu ý về phiên bản. Bản đặc tả hiện hành khi mình viết bài (2026-07-28) đã chuyển sang mô hình không trạng thái: mỗi request tự mang phiên bản giao thức và capability trong `_meta`, client hỏi khả năng của server qua `server/discover`. Các bản trước dùng bước bắt tay `initialize` và header `Mcp-Session-Id`; transport HTTP+SSE của bản 2024-11-05 đã bị đánh dấu deprecated. Hướng dẫn cũ khác với SDK mới là vì vậy.

Điều MCP không giải quyết: chất lượng tool. Một server bọc 80 endpoint thành 80 tool vẫn là 80 tool viết cho lập trình viên, chỉ khác là giờ ai cũng cài được trong một phút.

## Khi nối nhiều server: phình context và các lối ra

Cách triển khai ngây thơ là host gọi `tools/list` trên mọi server rồi nhét toàn bộ định nghĩa vào prompt từ lượt đầu. Với vài tool thì ổn. Với năm bảy server, định nghĩa có thể chiếm phần lớn context trước khi model đọc câu hỏi. Vấn đề thứ hai là kết quả trung gian: muốn chuyển một biên bản họp từ Google Drive sang Salesforce bằng tool call trực tiếp, nội dung biên bản đi qua model hai lần, một lần khi đọc và một lần khi model chép nó vào tham số của lệnh ghi.

### Lối ra 1: tool search và nạp trễ

Ý tưởng: model chỉ thấy một tool tìm kiếm và vài tool dùng thường xuyên; phần còn lại được đánh dấu `defer_loading: true` và chỉ vào context khi model tìm ra. Anthropic có sẵn hai biến thể (regex và BM25), OpenAI có `tool_search` cùng khái niệm namespace để nhóm tool theo miền, và trang hướng dẫn cho client của MCP mô tả cùng pattern ở phía host dưới tên progressive discovery, gợi ý chuyển sang chế độ này khi định nghĩa tool vượt khoảng 1-5% context window.

Số liệu Anthropic công bố: trong ví dụ của họ, context dùng trước khi bắt đầu việc giảm từ khoảng 77.000 xuống 8.700 token. Trên bộ eval MCP nội bộ, độ chính xác của Opus 4 tăng từ 49% lên 74%, của Opus 4.5 từ 79,5% lên 88,1%. Đây là số nội bộ, không kèm phương pháp đầy đủ, nên coi là chỉ dấu về hướng chứ không phải mức bạn sẽ đạt.

Giới hạn cần biết: thêm một bước tìm trước khi gọi tool nên tăng độ trễ. Với bản của Anthropic, bạn vẫn gửi toàn bộ định nghĩa trong mỗi request (chúng chỉ không vào context) và mỗi lần tìm mặc định trả về 5 tool. Tìm kiếm chạy trên tên và mô tả, nên tiền tố nhất quán và từ khóa trong mô tả trở thành điều kiện để tool được tìm thấy. Agent có dưới 10 tool và đều dùng thường xuyên thì không cần.

### Lối ra 2: cho agent viết code gọi tool

Thay vì gọi từng tool rồi nhận từng kết quả, host biến các MCP tool thành một API có kiểu (ví dụ một cây file TypeScript, mỗi tool một file) và cho model một tool duy nhất để chạy code trong sandbox. Model đọc đúng file nó cần, viết một đoạn script gọi nhiều tool, lọc và ghép dữ liệu ngay trong sandbox, rồi chỉ in ra phần tóm tắt.

```ts
// Code do model viết, chạy trong sandbox
const logs = await logging_getLogs({ level: "error", since: Date.now() - 3600000 });
const unique = new Map(logs.entries.map((l) => [l.message, l]));
for (const [message, log] of unique) {
  await ticketing_createIssue({ title: `Error: ${message}`, priority: "high" });
}
console.log(`Đã tạo ${unique.size} ticket từ ${logs.entries.length} dòng log`);
```

Hàng nghìn dòng log không vào context; model chỉ thấy một dòng tổng kết, và vòng lặp chạy bằng code thay vì mỗi bước một lượt suy luận. Cloudflare, bên gọi cách này là Code Mode, lập luận rằng model đã thấy rất nhiều TypeScript thật trong dữ liệu huấn luyện nhưng tương đối ít ví dụ tool call, nên viết code gọi API là việc nó làm tốt hơn. Anthropic đưa ví dụ giảm từ 150.000 xuống 2.000 token (98,7%); đó là một kịch bản minh họa trong bài của họ. Con số từ thử nghiệm thực mà họ công bố khiêm tốn hơn: tính năng programmatic tool calling giảm trung bình 37% token trên tác vụ nghiên cứu phức tạp.

Cái giá là một bề mặt tấn công mới. Chạy code do model sinh ra đòi sandbox không có mạng trực tiếp, giới hạn tài nguyên, và một lớp broker ở host giữ thông tin xác thực rồi kiểm tra từng lệnh gọi. Duyệt một script không có nghĩa là duyệt mọi lệnh nó sẽ gọi lúc chạy. Dữ liệu trả về từ server này là đầu vào không đáng tin với server kia. Nếu bạn chưa có hạ tầng sandbox, lối ra 1 rẻ hơn nhiều.

## Agent Skills và progressive disclosure

Skill giải một bài toán khác với tool: không phải "agent làm được gì" mà "agent biết làm việc này theo cách của bạn như thế nào". Một skill là một thư mục có file `SKILL.md` (frontmatter YAML với `name` và `description`, bên dưới là hướng dẫn) cùng script và tài liệu tham chiếu tùy chọn.

```text
bao-cao-doanh-thu/
├── SKILL.md            # name, description + quy trình
├── references/
│   └── schema.md       # chỉ đọc khi cần viết truy vấn
└── scripts/
    └── tong_hop.py     # chạy, không cần đọc mã
```

Skill được nạp theo ba tầng, gọi là progressive disclosure (tiết lộ dần):

| Tầng | Nạp khi nào | Chi phí |
| --- | --- | --- |
| Metadata (`name`, `description`) | Luôn có trong system prompt | Khoảng 100 token mỗi skill |
| Thân `SKILL.md` | Khi agent thấy tác vụ khớp mô tả | Khuyến nghị dưới 5.000 token, dưới 500 dòng |
| File tham chiếu, script | Khi hướng dẫn trỏ tới | Không tốn gì cho tới khi đọc; script chỉ tốn phần output |

Đây là cùng ý tưởng nạp trễ như tool search nhưng làm ở tầng hệ thống file: agent có sẵn tool đọc file và chạy lệnh, nên "tìm kiếm" chỉ là đọc mô tả rồi mở file. Điều kiện đi kèm là agent phải có hệ thống file và quyền chạy code.

Hai hệ quả thực hành. Thứ nhất, `description` là cò kích hoạt duy nhất, phải nói cả skill làm gì lẫn khi nào dùng, kèm từ khóa người dùng hay nhắc (giới hạn 1.024 ký tự). Mô tả kiểu "Hỗ trợ xử lý PDF" gần như đảm bảo skill không được gọi đúng lúc. Thứ hai, việc gì cần kết quả lặp lại chính xác (parse file, tính toán, kiểm tra định dạng) thì đưa vào script. Code cho kết quả giống nhau mỗi lần, và mã nguồn của script không vào context.

Về an toàn, coi việc cài skill như cài phần mềm. Skill là hướng dẫn cộng code chạy với quyền của agent, nên một skill độc hại có thể bảo agent gửi dữ liệu ra ngoài. Đọc hết các file, nhất là script và mọi chỉ dẫn gọi tới địa chỉ bên ngoài, trước khi dùng skill không phải do bạn viết.

## Chọn gì: MCP, Skill hay CLI thuần

Lựa chọn thứ ba hay bị quên: không thêm gì cả, để agent dùng công cụ dòng lệnh sẵn có qua tool chạy shell. Lập luận ở đây là suy luận của mình, mở rộng từ lập luận của Cloudflare về code: model đã thấy `git`, `gh`, `curl`, `jq` rất nhiều lần, các lệnh đó có `--help` để tự tra, và chúng tốn 0 token định nghĩa.

| Tiêu chí | CLI thuần | MCP server | Skill |
| --- | --- | --- | --- |
| Bản chất | Lệnh có sẵn, gọi qua shell | Tool có schema, qua giao thức | Hướng dẫn + script trên hệ thống file |
| Chi phí context thường trực | Gần 0 | Toàn bộ định nghĩa, trừ khi nạp trễ | Khoảng 100 token mỗi skill |
| Cần ở phía agent | Shell, tốt nhất có sandbox | Host hỗ trợ MCP | Hệ thống file và chạy code |
| Xác thực và phân quyền | Theo máy đang chạy, khó giới hạn từng thao tác | Theo từng server, có OAuth cho server từ xa | Thừa hưởng quyền của agent |
| Hợp nhất khi | Công cụ phổ biến, agent chạy trên máy lập trình viên | Dịch vụ từ xa, agent không có shell, cần kiểm soát từng thao tác | Quy trình và tri thức riêng của đội |
| Điểm yếu | Output viết cho người, dễ dài; lệnh phá hoại khó chặn | Phình context, chất lượng tùy người viết server | Phụ thuộc mô tả để kích hoạt; rủi ro từ skill lạ |

Ba thứ này không loại trừ nhau. Một tổ hợp hợp lý: MCP cho kết nối tới hệ thống cần xác thực, skill để dạy quy trình dùng các tool đó, CLI cho những thứ model vốn đã thạo.

## Checklist trước khi phát hành một tool

- Tên có tiền tố dịch vụ, không trùng nghĩa với tool khác.
- Mô tả trả lời đủ: làm gì, khi nào dùng, khi nào không, trả về gì, giới hạn gì.
- Mỗi tham số có mô tả, định dạng và ví dụ; tập giá trị đóng dùng `enum`.
- Có giá trị mặc định cho phân trang và mức chi tiết; kết quả bị cắt thì nói rõ cách lấy tiếp.
- Kết quả không chứa trường mà agent không dùng để quyết định bước sau.
- Mọi lỗi có ba phần: sai gì, đúng thì thế nào, làm gì tiếp.
- Thao tác ghi hoặc xóa được đánh dấu rõ và có bước xác nhận ở harness.
- Đã chạy ít nhất một bộ tác vụ thực tế và đọc transcript, không chỉ nhìn tỉ lệ đạt.
- Đã đếm token của toàn bộ định nghĩa và quyết định tool nào nạp sẵn, tool nào nạp trễ.

## Điều còn mở / chưa chắc

- Gọi tool trực tiếp hay viết code gọi tool sẽ thành mặc định? Anthropic và Cloudflare đều đẩy hướng code, nhưng nó đòi sandbox mà nhiều đội chưa có. Mình chưa thấy benchmark độc lập so sánh hai cách trên cùng bộ tác vụ.
- Ngưỡng "quá nhiều tool" là bao nhiêu? 20 theo OpenAI, 30-50 theo Anthropic, và cả hai đều phụ thuộc model. Con số của bạn chỉ có khi tự đo.
- Skill có thay MCP không? Hai thứ giải hai việc khác nhau, nhưng ranh giới đang mờ đi khi skill có thể bọc script gọi API. Với dịch vụ cần xác thực theo từng người dùng, mình nghĩ MCP vẫn cần, song đây là nhận định chứ chưa phải điều đã được chứng minh.
- Đặc tả MCP còn đổi nhanh. Bản 2026-07-28 thay cả mô hình kết nối, nên chi tiết giao thức trong bài này có thể cũ đi sau vài tháng.
- Phần lớn số liệu trong bài đến từ chính các nhà cung cấp, đo trên model và bộ eval của họ.

## Nguồn đọc thêm

- [Writing effective tools for agents](https://www.anthropic.com/engineering/writing-tools-for-agents) (Anthropic): nguồn chính cho phần nguyên tắc viết tool, có quy trình eval cụ thể.
- [Define tools](https://platform.claude.com/docs/en/agents-and-tools/tool-use/define-tools) và [Handle tool calls](https://platform.claude.com/docs/en/agents-and-tools/tool-use/handle-tool-calls) (tài liệu Claude API): cấu trúc `tool_use`, `tool_result`, ví dụ mô tả tốt và xấu.
- [Function calling](https://developers.openai.com/api/docs/guides/function-calling) (tài liệu OpenAI): cùng cơ chế với tên gọi khác, phần strict mode và namespace đáng đọc.
- [Advanced tool use](https://www.anthropic.com/engineering/advanced-tool-use) và [Tool search tool](https://platform.claude.com/docs/en/agents-and-tools/tool-use/tool-search-tool) (Anthropic): số liệu về nạp trễ, programmatic tool calling và `input_examples`.
- [Code execution with MCP](https://www.anthropic.com/engineering/code-execution-with-mcp) (Anthropic) và [Code Mode](https://blog.cloudflare.com/code-mode/) (Cloudflare): hai bài cùng đi tới ý tưởng cho agent viết code gọi tool.
- [MCP: Architecture overview](https://modelcontextprotocol.io/docs/learn/architecture), đặc tả [stdio](https://modelcontextprotocol.io/specification/2026-07-28/basic/transports/stdio) và [Streamable HTTP](https://modelcontextprotocol.io/specification/2026-07-28/basic/transports/streamable-http): đọc để hiểu giao thức thật thay vì qua lớp SDK.
- [MCP: Client best practices](https://modelcontextprotocol.io/docs/2026-07-28/develop/clients/client-best-practices): progressive discovery và code mode nhìn từ phía người viết host, có phần bảo mật kỹ.
- [Equipping agents for the real world with Agent Skills](https://www.anthropic.com/engineering/equipping-agents-for-the-real-world-with-agent-skills) (Anthropic), [Agent Skills overview](https://platform.claude.com/docs/en/agents-and-tools/agent-skills/overview) và [đặc tả Agent Skills](https://agentskills.io/specification): ba tầng nạp và định dạng `SKILL.md`.
- [Building effective agents](https://www.anthropic.com/engineering/building-effective-agents) (Anthropic): phụ lục về agent-computer interface và ví dụ đường dẫn tuyệt đối.
