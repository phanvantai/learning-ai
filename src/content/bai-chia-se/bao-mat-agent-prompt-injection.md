---
title: "Bảo mật cho agent: prompt injection và phòng thủ ở tầng harness"
date: 2026-11-11
tags: [Bảo mật, Agent]
description: "Vì sao prompt injection chưa có lời giải triệt để ở tầng model, lethal trifecta, các đường rò rỉ dữ liệu, rủi ro MCP, và cách phòng thủ theo lớp: sandbox, allowlist egress, phê duyệt, design pattern. Kèm checklist threat-model cho agent nội bộ."
loai: chuyen-sau
---

SQL injection có lời giải gọn: parameterized query tách hẳn câu lệnh khỏi dữ liệu, cơ sở dữ liệu không bao giờ diễn giải dữ liệu như lệnh. LLM không có cơ chế tương đương. System prompt, câu hỏi của người dùng, nội dung trang web vừa tải về, kết quả tool đều nằm chung một chuỗi token, và model quyết định làm theo cái gì dựa trên nội dung chứ không dựa trên nguồn gốc. Simon Willison tóm lại: LLM không thể phân biệt một cách đáng tin cậy tầm quan trọng của chỉ dẫn dựa trên nơi chúng đến từ.

Với chatbot, hậu quả là một câu trả lời sai. Với agent có tool, hậu quả là hành động sai: đọc file, gửi thư, gọi API bằng quyền của bạn. Bài này nói về cách thiết kế để khi model bị lừa thì thiệt hại bị chặn ở tầng dưới. Bài thuần phòng thủ: mình mô tả cơ chế tấn công ở mức đủ hiểu để phòng, không đưa payload.

## Trực tiếp và gián tiếp

OWASP xếp prompt injection là rủi ro số một (LLM01) trong Top 10 cho ứng dụng LLM và chia hai loại:

- **Trực tiếp**: chính đầu vào của người dùng làm thay đổi hành vi model. Người gõ là kẻ tấn công (hoặc vô tình). Jailbreak thuộc nhóm này.
- **Gián tiếp**: model nhận đầu vào từ nguồn bên ngoài như trang web hay file, và nội dung đó chứa thứ làm đổi hành vi model. Người dùng là nạn nhân, kẻ tấn công ở chỗ khác.

Loại gián tiếp là mối lo chính của agent, vì bề mặt tấn công là mọi thứ agent đọc: trang web, email đến, issue trên GitHub, comment trong code, tên file, kết quả từ API bên thứ ba, bản ghi CRM do khách tự điền. Kẻ tấn công không cần truy cập hệ thống của bạn, chỉ cần đặt văn bản ở nơi agent sẽ đọc tới.

## Vì sao chưa có lời giải triệt để ở tầng model

Các nhà cung cấp có huấn luyện để model kháng injection tốt hơn, và có tiến bộ thật. Số liệu của chính họ cũng cho thấy giới hạn. Trong bài công bố tháng 11/2025 về phòng thủ cho agent dùng trình duyệt, Anthropic báo tỉ lệ tấn công thành công với Claude Opus 4.5 trước một attacker thích ứng nội bộ là 1,4% khi có lớp bảo vệ cải tiến (8,0% nếu chỉ có model). Họ viết luôn rằng 1% vẫn là rủi ro đáng kể, rằng prompt injection còn xa mới được giải quyết, và không agent trình duyệt nào miễn nhiễm.

Có ba lý do con số nhỏ đó chưa đủ.

**Bảo mật là bài toán đối kháng, không phải thống kê.** Kẻ tấn công không lấy mẫu ngẫu nhiên. Họ thử lại cho tới khi trúng, rồi dùng lại đúng cái trúng. Simon Willison nói về các sản phẩm guardrail quảng cáo chặn 95% tấn công: trong bảo mật ứng dụng web, 95% là điểm trượt.

**Phòng thủ được đánh giá bằng tấn công yếu.** Paper "The Attacker Moves Second" (tháng 10/2025, nhóm tác giả gồm người của OpenAI, Anthropic và Google DeepMind) thử 12 cơ chế phòng thủ đã công bố bằng tấn công thích ứng, tức là kẻ tấn công biết thiết kế phòng thủ và tối ưu để vượt qua. Tỉ lệ thành công trên 90% với phần lớn các phòng thủ, trong khi đa số từng báo cáo tỉ lệ gần 0. Theo bản tóm tắt của Simon Willison, vòng red-team bằng người thật đã vượt qua toàn bộ.

**Không có ranh giới kiểu dữ liệu.** Mọi cách "đánh dấu" nội dung không tin cậy (bọc thẻ, thêm cảnh báo) đều là thêm văn bản vào cùng kênh với văn bản tấn công. Chúng nâng chi phí tấn công, không tạo ra bảo đảm.

Kết luận thực hành: coi model là một thành phần có thể bị điều khiển bất cứ lúc nào sau khi đã đọc nội dung không tin cậy, và đặt các bảo đảm ở phần code tất định bao quanh nó.

## Lethal trifecta

Simon Willison đặt tên cho tổ hợp nguy hiểm nhất. Một agent có đủ ba thứ sau thì kẻ tấn công có thể lấy dữ liệu của bạn:

```text
        (1) Truy cập dữ liệu riêng tư
               /            \
              /   NGUY HIỂM  \
             /   khi đủ cả 3  \
(2) Tiếp xúc nội dung ------- (3) Có khả năng liên lạc
    không tin cậy                 ra bên ngoài
```

Cơ chế: nội dung không tin cậy (2) mang chỉ dẫn, model làm theo, đọc dữ liệu riêng (1), rồi gửi đi qua kênh ra ngoài (3). Bỏ một cạnh thì chuỗi đứt. Lời khuyên của ông cho người dùng cuối là tránh hẳn tổ hợp này, vì không thể trông vào guardrail.

Trifecta khó tránh vì nó hình thành do ghép tool. Từng tool riêng lẻ trông vô hại: một tool đọc email, một tool tìm web, một tool gọi HTTP. Ghép lại là đủ ba cạnh. Willison chỉ ra đây là vấn đề với MCP: nó khuyến khích người dùng trộn tool từ nhiều nguồn.

Meta mở rộng ý này thành "Agents Rule of Two" (bài công bố ngày 31/10/2025). Trong một phiên, agent chỉ nên có tối đa hai trong ba thuộc tính:

- [A] xử lý đầu vào không tin cậy;
- [B] truy cập hệ thống nhạy cảm hoặc dữ liệu riêng tư;
- [C] thay đổi trạng thái hoặc liên lạc ra ngoài.

Khác biệt so với trifecta nằm ở [C]: gồm cả thay đổi trạng thái, không riêng gửi dữ liệu đi. Nó bao luôn trường hợp agent bị lừa xóa hay sửa thứ gì đó. Nếu cần cả ba, Meta nói agent không được chạy tự động, tối thiểu phải có giám sát như phê duyệt của người, hoặc bắt đầu phiên mới với context sạch. Họ cũng nói rõ quy tắc này bổ sung chứ không thay cho least privilege.

Willison đánh giá đây là hướng dẫn thực dụng tốt nhất hiện có, kèm một phản biện: sơ đồ ban đầu ghi tổ hợp [A]+[C] là "an toàn", trong khi đầu vào không tin cậy cộng quyền thay đổi trạng thái vẫn gây hại được. Meta sau đó đổi nhãn thành "rủi ro thấp hơn".

## Dữ liệu đi ra ngoài bằng đường nào

Cạnh (3) rộng hơn nhiều người nghĩ. Bất cứ thứ gì khiến một request rời hệ thống với nội dung do model quyết định đều là kênh rò rỉ.

**Ảnh markdown.** Đây là đường kinh điển. Giao diện chat render markdown; nếu model xuất ra một ảnh trỏ tới máy chủ của kẻ tấn công với dữ liệu nhét trong URL, trình duyệt của nạn nhân tự tải ảnh và dữ liệu nằm trong log của máy chủ đó. Không cần ai bấm gì. Cách chặn chuẩn: không render ảnh từ domain ngoài, hoặc dùng Content Security Policy với allowlist domain ảnh.

**Allowlist cũng thủng.** Loạt sự cố Simon Willison ghi lại cho thấy allowlist domain chỉ tốt bằng domain yếu nhất trong danh sách:

| Sự cố (theo ghi chép của Simon Willison) | Vì sao allowlist không cứu được |
| --- | --- |
| Microsoft 365 Copilot "EchoLeak" (6/2025) | Dùng cú pháp ảnh markdown dạng reference mà bộ lọc bỏ sót, cộng một open redirect trên domain nằm trong allowlist CSP |
| GitLab Duo (5/2025) | Chỉ dẫn ẩn trong code dự án công khai khiến trợ lý lấy nội dung merge request riêng tư rồi rò qua thẻ ảnh |
| Salesforce AgentForce (9/2025) | CSP còn liệt kê một domain đã hết hạn, nhà nghiên cứu đăng ký lại domain đó |
| Superhuman AI (1/2026) | CSP cho phép ảnh từ một domain dịch vụ biểu mẫu công cộng, nơi nhận và lưu dữ liệu gửi qua GET |

**Link và link preview.** Link có dữ liệu trong URL cần người bấm, nên yếu hơn ảnh. Nhưng nhiều ứng dụng chat tự tải URL để dựng preview (unfurl), biến link thành kênh không cần bấm. Agent đăng tin vào kênh chat thì nên tắt unfurl.

**Tool có tác dụng phụ ra ngoài.** Gửi email, đăng comment, tạo issue ở repo công khai, gọi webhook, và cả tool "chỉ đọc" như tải một URL (URL chính là dữ liệu gửi đi). Với agent chạy code còn có `curl`, tra cứu DNS, `git push`, cài package từ registry. Vì vậy với coding agent, chặn ở từng tool là không đủ, phải chặn ở tầng mạng.

## Rủi ro riêng của MCP

MCP (Model Context Protocol) chuẩn hóa cách nối tool vào agent, nên cũng chuẩn hóa luôn vài lớp rủi ro. Thuật ngữ dưới đây phần lớn theo bài tổng hợp của Simon Willison về nghiên cứu của Invariant Labs.

**Tool poisoning.** Mô tả tool là văn bản đi thẳng vào context của model, và người dùng thường không nhìn thấy đầy đủ. Một server độc hại có thể giấu chỉ dẫn trong mô tả của một tool trông vô hại. Mô tả tool vì thế phải được coi là nội dung không tin cậy, ngang với trang web.

**Rug pull.** Server có thể đổi định nghĩa tool sau khi bạn đã duyệt. Phòng: ghim phiên bản server, lưu hash định nghĩa tool lúc duyệt, và yêu cầu duyệt lại khi hash đổi.

**Tool shadowing.** Khi nhiều server cùng nối vào một agent, một server độc hại có thể chen vào hoặc ghi đè các lời gọi vốn dành cho server tin cậy. Trong context không có ranh giới tin cậy giữa các server.

**Confused deputy.** Nghĩa gốc: một chương trình có quyền bị bên không có quyền lừa dùng quyền đó hộ. Agent nào cũng là một deputy như vậy. Tài liệu bảo mật của MCP dùng từ này cho một lỗ hổng OAuth cụ thể: server MCP làm proxy tới API bên thứ ba bằng một client ID tĩnh, lại cho client đăng ký động, nên kẻ tấn công có thể lợi dụng cookie đồng ý sẵn có của người dùng để lấy mã ủy quyền mà không qua màn hình đồng ý. Yêu cầu của đặc tả: server proxy phải (MUST) thực hiện đồng ý theo từng client trước khi chuyển tiếp. Cùng nhóm là "token passthrough": server không được (MUST NOT) chấp nhận token không được cấp cho chính nó.

**Server chạy cục bộ.** Một server MCP cục bộ là một tiến trình chạy bằng quyền của bạn trên máy bạn; cài nó tương đương chạy mã của người lạ. Đặc tả khuyến nghị client hiển thị đầy đủ lệnh sẽ chạy và nên chạy server trong sandbox với quyền tối thiểu.

## Phòng thủ theo lớp ở tầng harness

Nguyên tắc xuyên suốt: mỗi lớp giả định lớp phía trên đã thủng.

### Sandbox filesystem và network, phải có cả hai

Bài của Anthropic về sandbox cho Claude Code nêu lý do cần đủ hai lớp cô lập: thiếu cô lập mạng, agent bị chiếm quyền có thể gửi đi các file nhạy cảm như SSH key; thiếu cô lập filesystem, agent có thể thoát khỏi sandbox rồi giành lại quyền truy cập mạng. Hai lớp vá lỗ của nhau.

Chi tiết triển khai đáng học:

- Cô lập ở tầng hệ điều hành (bubblewrap trên Linux, seatbelt trên macOS), nên áp dụng cho cả script và tiến trình con mà agent sinh ra. Kiểm tra ở tầng tool call không bao được những thứ đó.
- Mạng đi qua một proxy chạy ngoài sandbox. Proxy là nơi áp allowlist domain và hỏi người dùng khi có domain mới.
- Credential không nằm trong sandbox. Ở bản chạy trên web, git client trong sandbox chỉ giữ một credential phạm vi hẹp. Một proxy bên ngoài kiểm tra thao tác git (ví dụ chỉ được push lên đúng nhánh đã cấu hình) rồi mới gắn token thật trước khi chuyển tới GitHub.

Điểm cuối đáng bê nguyên: thứ agent không cầm thì agent không làm lộ được. Thay vì đưa API key vào biến môi trường của agent, đặt key ở proxy và để proxy gắn vào request tới đúng host.

### Allowlist egress

Chặn mặc định mọi kết nối ra, chỉ mở đích cần. Từ các sự cố ở trên, vài quy tắc khi lập allowlist:

- Liệt kê host chính xác, không wildcard theo domain cha.
- Với mỗi host, hỏi: bên ngoài có ghi được dữ liệu lên đây rồi đọc lại không? Kho lưu trữ công cộng, dịch vụ paste, biểu mẫu, webhook đều thuộc loại này. Open redirect trên host đó cũng tính.
- Phân biệt đọc với ghi nếu proxy làm được: cho GET tới registry package, không cho POST.
- Rà định kỳ, vì domain hết hạn trong allowlist là lỗ hổng có thật (ca AgentForce).

Allowlist không chặn rò rỉ qua một host hợp lệ mà agent có quyền ghi: agent tạo được issue công khai trên nền tảng git của bạn thì nền tảng đó là kênh ra.

### Permission prompt và mệt mỏi phê duyệt

Hỏi người dùng trước mỗi hành động nghe an toàn nhất, thực tế thì không. Anthropic gọi hiện tượng này là "approval fatigue": hỏi liên tục khiến người dùng thôi chú ý vào thứ mình đang duyệt. Con số họ đưa ra từ sử dụng nội bộ: sandbox giảm 84% số lần hỏi quyền. Paper CaMeL nêu cùng lo ngại từ góc khác: người dùng bị hỏi nhiều sẽ trả lời "có" cho mọi thứ.

Hệ quả thiết kế: coi lượt phê duyệt là tài nguyên khan hiếm.

- Việc nằm gọn trong sandbox (đọc, sửa file trong thư mục làm việc, chạy test) thì không hỏi.
- Chỉ hỏi ở ranh giới: ra ngoài sandbox, tới host mới, hành động không đảo ngược.
- Lời hỏi phải hiện tham số thật và đầy đủ (lệnh nguyên văn, người nhận, nội dung gửi đi), không phải bản model tóm tắt. Một trong các ca Willison ghi lại giấu dữ liệu bị lấy ở tít bên phải một trường văn bản, nơi giao diện không hiện thanh cuộn ngang.

### Tách quyền đọc và ghi

Token của agent nên hẹp hơn quyền của người dùng, và đọc với ghi dùng credential khác nhau. Agent phân tích dữ liệu dùng tài khoản DB chỉ đọc, giới hạn theo bảng. Token git chỉ cho một repo, không có quyền quản trị, không đọc được secret của CI.

Tách cả theo phiên: phiên đọc nội dung không tin cậy thì không gắn tool ghi vào hệ thống nhạy cảm.

### Human-in-the-loop cho hành động không đảo ngược

Phân loại tool theo khả năng hoàn tác, không theo cảm giác nguy hiểm:

| Loại hành động | Ví dụ | Xử lý |
| --- | --- | --- |
| Đọc trong phạm vi | Đọc file trong workspace, query DB chỉ đọc | Tự động |
| Ghi, hoàn tác được | Sửa file có git, tạo bản nháp, mở PR | Tự động, có log |
| Ghi, khó hoàn tác | Gửi email, đăng tin, merge, chạy migration | Người duyệt, thấy nội dung đầy đủ |
| Không hoàn tác được | Chuyển tiền, xóa dữ liệu, thu hồi quyền | Người duyệt, cân nhắc không trao tool này cho agent |

Cách giảm số lần hỏi mà không giảm an toàn: biến hành động khó hoàn tác thành hoàn tác được. Agent tạo bản nháp thay vì gửi, mở PR thay vì push thẳng lên nhánh chính, ghi vào bảng chờ duyệt thay vì bảng thật.

### Một cổng kiểm soát tool tối thiểu

Đoạn dưới minh họa ý "context đã nhiễm thì thu hẹp quyền". Nó là khung sườn, chưa đủ cho production.

```python
from dataclasses import dataclass, field

UNTRUSTED_SOURCES = {"web_fetch", "read_email", "read_issue", "mcp:third_party"}
EGRESS_TOOLS      = {"send_email", "http_request", "web_fetch", "post_comment", "git_push"}
IRREVERSIBLE      = {"delete_records", "merge_pr", "send_email"}

@dataclass
class Session:
    tainted: bool = False                 # context đã chứa nội dung không tin cậy chưa
    log: list = field(default_factory=list)

def gate(session: Session, tool: str, args: dict, ask_human) -> bool:
    """Chạy trong harness, ngoài tầm với của model. Trả về True nếu cho phép."""
    if tool in IRREVERSIBLE:
        decision = ask_human(tool, args)          # hiện tham số thật, không tóm tắt
    elif tool in EGRESS_TOOLS and session.tainted:
        decision = ask_human(tool, args)          # trifecta sắp khép: người quyết
    else:
        decision = True
    session.log.append((tool, args, session.tainted, decision))
    return decision

def after_tool(session: Session, tool: str) -> None:
    if tool in UNTRUSTED_SOURCES:
        session.tainted = True                    # không có đường gỡ cờ trong cùng phiên
```

Quyết định nằm trong code tất định, model không tự gỡ được cờ `tainted`. Cờ chỉ đi một chiều: muốn gửi ra ngoài mà không bị hỏi thì bắt đầu phiên mới với context sạch. Đánh dấu cả phiên như vậy thô nhưng dễ kiểm chứng. CaMeL ở phần dưới theo dõi tinh hơn, tới từng giá trị.

## Design pattern: ràng buộc agent sau khi nó đọc nội dung bẩn

Paper "Design Patterns for Securing LLM Agents against Prompt Injections" (tháng 6/2025) khởi đi từ một nguyên tắc: khi agent đã nạp đầu vào không tin cậy, nó phải bị ràng buộc sao cho đầu vào đó không thể kích hoạt hành động gây hậu quả. Paper đưa ra sáu pattern, mỗi cái đổi một phần năng lực lấy bảo đảm.

| Pattern | Cơ chế | Cái giá |
| --- | --- | --- |
| Action-Selector | Model chỉ làm công tắc chọn một hành động định sẵn, kết quả hành động không quay lại model | Chỉ làm được việc đã lường trước |
| Plan-Then-Execute | Chốt chuỗi tool call trước khi đọc dữ liệu không tin cậy | Injection vẫn đổi được tham số của các bước đã lên kế hoạch; khó với việc cần quyết định theo kết quả |
| LLM Map-Reduce | Mỗi mẩu dữ liệu bẩn do một instance cô lập, không tool, xử lý; đầu ra bị ràng buộc rồi mới gộp | Mất suy luận chéo giữa các mẩu; mẩu độc vẫn làm lệch kết quả của chính nó |
| Dual LLM | LLM có quyền không bao giờ đọc dữ liệu bẩn; LLM cách ly đọc dữ liệu bẩn nhưng không có tool | Phức tạp; LLM cách ly vẫn bị điều khiển được đầu ra |
| Code-Then-Execute | Agent viết một chương trình gọi tool; chương trình chạy trên dữ liệu bẩn | Luồng điều khiển cố định, giá trị vẫn bị ảnh hưởng; đòi model viết code tin cậy |
| Context-Minimization | Gỡ prompt của người dùng khỏi context sau khi nó đã được dùng để chọn hành động | Câu trả lời cuối không phản ứng được với lời người dùng |

**Dual LLM** là đề xuất của Simon Willison từ 2023. Một controller viết bằng code thường điều phối hai LLM và giữ kết quả của LLM cách ly trong biến; LLM có quyền chỉ thấy tên biến, không thấy nội dung. Chính ông nêu giới hạn: phức tạp, dễ sơ suất làm rò văn bản bẩn sang phía có quyền, và không chống được việc người dùng bị dụ tự dán nội dung độc vào.

**CaMeL** (Google DeepMind, tháng 3/2025) vá một lỗ của Dual LLM. Trong Dual LLM, khi LLM có quyền nhờ LLM cách ly trích địa chỉ email người nhận từ một bức thư, kẻ tấn công kiểm soát bức thư sẽ kiểm soát luôn giá trị trả về, và tài liệu được gửi tới sai người dù luồng điều khiển không hề đổi. CaMeL xử lý thế này:

1. LLM có quyền chỉ thấy yêu cầu của người dùng và viết ra một chương trình bằng một tập con của Python.
2. Một interpreter riêng chạy chương trình đó và theo dõi mỗi giá trị bắt nguồn từ đâu. Giá trị trích từ email không tin cậy mang nhãn không tin cậy, nhãn lan theo mọi phép tính.
3. Trước mỗi tool call, interpreter kiểm tra chính sách dựa trên các nhãn đó. Ví dụ: tool gửi email chỉ chạy nếu người nhận đến từ nguồn tin cậy, nếu không thì hỏi người dùng.

Trên benchmark AgentDojo, CaMeL giải được 77% task với bảo đảm an toàn chứng minh được, so với 84% của hệ thống không phòng thủ. Ngoài 7 điểm đó còn các cái giá chính paper nêu: phải có người viết và bảo trì chính sách bảo mật, người dùng vẫn có thể bị hỏi nhiều tới mức duyệt bừa, và prompt injection chưa được giải quyết hoàn toàn.

Chọn pattern nào? Mình nghĩ nên đi từ cái hẹp nhất còn đáp ứng được yêu cầu. Chatbot tra cứu đơn hàng hợp với Action-Selector. Pipeline tóm tắt hàng loạt tài liệu ngoài hợp với Map-Reduce. Trợ lý đa năng trên email và lịch là nơi cần thứ như CaMeL, và cũng là nơi khó nhất.

## Giới hạn của classifier và guardrail

Bộ lọc đầu vào (phát hiện injection) và đầu ra (phát hiện rò rỉ) đáng có, với kỳ vọng đúng. Chúng cũng là model, nên kết quả của "The Attacker Moves Second" áp dụng thẳng vào chúng. Chúng có false positive: tài liệu bảo mật, email nói về phishing, bài viết như bài này đều chứa văn bản giống tấn công, nên ngưỡng chặt thì hỏng tính năng, ngưỡng lỏng thì lọt. Bộ lọc đầu ra theo mẫu (tìm chuỗi giống API key) bị vượt bằng mã hóa hay chia nhỏ.

Vai trò hợp lý: giảm lượng tấn công đại trà, tạo tín hiệu cho giám sát, nâng chi phí cho kẻ tấn công. Không dùng làm lớp duy nhất đứng trước một hành động không đảo ngược. Anthropic cũng trình bày classifier của họ như một lớp bên cạnh huấn luyện model và red-team.

## Checklist threat-model cho một agent nội bộ

Giả sử bạn sắp triển khai một agent nội bộ, ví dụ agent đọc ticket hỗ trợ, tra hệ thống đơn hàng và soạn trả lời. Đi qua danh sách này trước khi bật.

**1. Kiểm kê ba cạnh**

- Liệt kê mọi nguồn agent đọc. Đánh dấu nguồn nào người ngoài tổ chức ghi vào được (ticket, email, trang web, tên file, trường ghi chú trong CRM, mô tả tool của MCP server bên thứ ba).
- Liệt kê mọi dữ liệu và hệ thống agent chạm tới. Agent có thấy dữ liệu của khách A khi đang xử lý cho khách B không?
- Liệt kê mọi đường ra: tool gửi tin, tool gọi HTTP, render markdown ở giao diện, link preview, quyền ghi lên nơi công khai, bộ nhớ dài hạn.
- Có phiên nào hội đủ cả ba không? Nếu có: bỏ cạnh nào, hoặc đặt người duyệt ở đâu?

**2. Quyền**

- Credential của agent có hẹp hơn quyền của người vận hành không? Đọc và ghi đã tách chưa?
- Secret có nằm trong môi trường agent chạy không (biến môi trường, file cấu hình, lịch sử shell)? Chuyển được ra proxy không?
- Bỏ được tool nào mà sản phẩm vẫn chạy?

**3. Cô lập**

- Agent chạy code hay lệnh shell thì đã có sandbox filesystem lẫn network ở tầng hệ điều hành chưa?
- Egress mặc định chặn chưa? Mỗi host trong allowlist đã được hỏi câu "bên ngoài ghi vào rồi đọc lại được không" chưa?
- Giao diện hiển thị đầu ra có render ảnh hay tự tải URL từ domain ngoài không?

**4. Phê duyệt**

- Hành động nào không đảo ngược? Mỗi cái đã có người duyệt, và người duyệt thấy tham số nguyên văn chưa?
- Mỗi phiên hỏi bao nhiêu lần? Nhiều thì người dùng sẽ duyệt theo quán tính; chuyển bớt sang sandbox hoặc dạng bản nháp.

**5. MCP và chuỗi cung ứng**

- Mỗi MCP server: ai viết, chạy bằng quyền gì, đã ghim phiên bản chưa? Có phát hiện khi định nghĩa tool thay đổi không?

**6. Phát hiện và ứng phó**

- Mọi tool call có được log kèm tham số, đủ để dựng lại sự việc không?
- Có cảnh báo khi tool gửi ra ngoài được gọi ngay sau khi đọc nguồn không tin cậy, hoặc khi xuất hiện host mới không?
- Thu hồi credential của agent mất bao lâu? Đã thử chưa?

## Điều còn mở, chưa chắc

- **Model có bao giờ tự kháng được không.** Trong số liệu của Anthropic, model mới có tỉ lệ bị tấn công thành công thấp hơn model cũ, nhưng "The Attacker Moves Second" cho thấy phòng thủ ở tầng model liên tục bị tấn công thích ứng vượt qua. Simon Willison nói ông không chia sẻ sự lạc quan rằng phòng thủ đáng tin sẽ sớm có. Các nhà cung cấp vẫn đầu tư mạnh vào hướng này. Mình không biết ai đúng, và thiết kế hệ thống hôm nay nên giả định là chưa.
- **CaMeL và các pattern có dùng được cho agent đa năng không.** Bảo đảm của chúng đến từ việc hạn chế agent. Với coding agent chạy hàng trăm bước, quyết định dựa trên thứ vừa đọc, Plan-Then-Execute hay Code-Then-Execute thuần túy không khớp. Phần lớn sản phẩm coding agent hiện dựa vào sandbox và phê duyệt thay vì các pattern này (nhận định của mình, chưa có khảo sát).
- **Human-in-the-loop có bền không.** Meta, đặc tả MCP và CaMeL đều dựa vào người duyệt ở điểm cuối, trong khi Anthropic và paper CaMeL thừa nhận người duyệt sẽ mỏi. Chưa có lời giải tốt cho mâu thuẫn này ngoài việc hỏi ít đi.

## Nguồn đọc thêm

- [The lethal trifecta for AI agents](https://simonwillison.net/2025/Jun/16/the-lethal-trifecta/) (Simon Willison). Bài gốc của khái niệm, ngắn, đọc đầu tiên.
- [Loạt bài gắn thẻ exfiltration-attacks](https://simonwillison.net/tags/exfiltration-attacks/) (Simon Willison). Hồ sơ các sự cố thật, cho thấy allowlist thủng theo những cách nào.
- [Model Context Protocol has prompt injection security problems](https://simonwillison.net/2025/Apr/9/mcp-prompt-injection/) (Simon Willison). Tool poisoning, rug pull, shadowing.
- [MCP Security Best Practices](https://modelcontextprotocol.io/specification/2025-06-18/basic/security_best_practices) (đặc tả MCP). Confused deputy trong OAuth, token passthrough, server cục bộ, với các yêu cầu MUST và SHOULD cụ thể.
- [LLM01: Prompt Injection](https://genai.owasp.org/llmrisk/llm01-prompt-injection/) (OWASP). Định nghĩa chuẩn và danh sách biện pháp giảm thiểu, tiện để đối chiếu khi review.
- [Beyond permission prompts: making Claude Code more secure and autonomous](https://www.anthropic.com/engineering/claude-code-sandboxing) (Anthropic). Thiết kế sandbox hai lớp và proxy giữ credential, kèm con số 84%.
- [Mitigating the risk of prompt injections in browser use](https://www.anthropic.com/research/prompt-injection-defenses) (Anthropic). Số liệu tấn công thành công theo model và lời thừa nhận vấn đề chưa được giải quyết.
- [Agents Rule of Two: A Practical Approach to AI Agent Security](https://ai.meta.com/blog/practical-ai-agent-security/) (Meta). Quy tắc hai trên ba với ví dụ cấu hình cho từng loại agent.
- [The Attacker Moves Second](https://arxiv.org/abs/2510.09023) (Nasr và cộng sự, 2025), đọc kèm [bài điểm hai paper của Simon Willison](https://simonwillison.net/2025/Nov/2/new-prompt-injection-papers/). Lý do không nên tin tỉ lệ chặn do bên làm phòng thủ tự báo cáo, và phản biện với Rule of Two.
- [Design Patterns for Securing LLM Agents against Prompt Injections](https://arxiv.org/abs/2506.08837) (Beurer-Kellner và cộng sự, 2025). Sáu pattern với phân tích đánh đổi và case study.
- [The Dual LLM pattern for building AI assistants that can resist prompt injection](https://simonwillison.net/2023/Apr/25/dual-llm-pattern/) (Simon Willison). Đề xuất gốc, có phần tự phê bình.
- [Defeating Prompt Injections by Design](https://arxiv.org/abs/2503.18813) (Debenedetti và cộng sự, 2025) và [bài giải thích CaMeL của Simon Willison](https://simonwillison.net/2025/Apr/11/camel/). Paper CaMeL và bản đọc dễ hơn.
