---
title: "Bảo mật cho agent: prompt injection và phòng thủ ở tầng harness"
date: 2026-11-11
tags: [Bảo mật, Agent]
description: "Vì sao prompt injection chưa có lời giải triệt để ở tầng model, lethal trifecta, các đường rò rỉ dữ liệu, rủi ro MCP, và cách phòng thủ theo lớp: sandbox, allowlist egress, phê duyệt, design pattern. Kèm checklist threat-model cho agent nội bộ."
loai: chuyen-sau
---

SQL injection có lời giải gọn: parameterized query tách hẳn câu lệnh khỏi dữ liệu, cơ sở dữ liệu không bao giờ diễn giải dữ liệu như lệnh. LLM không có cơ chế tương đương. System prompt, câu hỏi của người dùng, nội dung trang web vừa tải về, kết quả tool đều nằm chung một chuỗi token, và model quyết định làm theo cái gì dựa trên nội dung chứ không dựa trên nguồn gốc. Simon Willison tóm lại: LLM không phân biệt được một cách đáng tin cậy tầm quan trọng của chỉ dẫn theo nơi chúng xuất phát.

Với chatbot, hậu quả là một câu trả lời sai. Với agent có tool, hậu quả là hành động sai: đọc file, gửi thư, gọi API bằng quyền của bạn. Bài này nói về cách thiết kế để khi model bị lừa thì thiệt hại bị chặn ở tầng dưới. Bài thuần phòng thủ: cơ chế tấn công chỉ được mô tả ở mức đủ hiểu để phòng, không có payload. Nhận định nào là suy luận của mình chứ không phải của nguồn, mình ghi rõ.

## Trực tiếp và gián tiếp

OWASP xếp prompt injection là rủi ro số một (LLM01) trong Top 10 cho ứng dụng LLM và chia hai loại. **Trực tiếp**: chính đầu vào của người dùng làm đổi hành vi model, cố ý hoặc vô tình; jailbreak thuộc nhóm này. **Gián tiếp**: model nhận nội dung từ nguồn bên ngoài như trang web hay file, và nội dung đó làm đổi hành vi model; người dùng là nạn nhân, kẻ tấn công ở chỗ khác.

Loại gián tiếp là mối lo chính của agent, vì bề mặt tấn công là mọi thứ agent đọc: trang web, email đến, issue, comment trong code, tên file, kết quả từ API bên thứ ba, bản ghi CRM do khách tự điền. Kẻ tấn công chỉ cần đặt văn bản ở nơi agent sẽ đọc tới.

## Vì sao chưa có lời giải triệt để ở tầng model

Các nhà cung cấp có huấn luyện để model kháng injection tốt hơn, và số liệu của chính họ cho thấy cả tiến bộ lẫn giới hạn. Trong bài công bố tháng 11/2025 về phòng thủ cho agent dùng trình duyệt, Anthropic báo tỉ lệ tấn công thành công với Claude Opus 4.5 trước một attacker thích ứng nội bộ là 1,4% khi có lớp bảo vệ cải tiến (8,0% nếu chỉ có model), với 100 lần thử cho mỗi môi trường. Họ viết luôn rằng tỉ lệ 1% vẫn là rủi ro đáng kể, rằng prompt injection còn xa mới được giải quyết, và không agent trình duyệt nào miễn nhiễm.

Có ba lý do con số nhỏ đó chưa đủ. Hai lý do đầu có nguồn, lý do thứ ba là lập luận của mình từ cơ chế nêu ở đầu bài.

**Bảo mật là bài toán đối kháng.** Kẻ tấn công không lấy mẫu ngẫu nhiên: họ thử lại cho tới khi trúng, rồi dùng lại đúng cái trúng. Simon Willison nói về các sản phẩm guardrail quảng cáo chặn 95% tấn công: trong bảo mật ứng dụng web, 95% là điểm trượt.

**Phòng thủ được đánh giá bằng tấn công yếu.** Paper "The Attacker Moves Second" (tháng 10/2025; 14 tác giả, tác giả đầu là Milad Nasr, theo Simon Willison gồm người của OpenAI, Anthropic và Google DeepMind) thử 12 cơ chế phòng thủ đã công bố bằng tấn công thích ứng, tức là kẻ tấn công biết thiết kế phòng thủ và tối ưu để vượt qua. Tỉ lệ thành công trên 90% với phần lớn các phòng thủ, trong khi đa số từng báo cáo tỉ lệ gần 0. Theo bản tóm tắt của Willison, vòng red-team bằng người (một cuộc thi trực tuyến 500 người tham gia) vượt qua toàn bộ.

**Không có ranh giới kiểu dữ liệu.** Mọi cách đánh dấu nội dung không tin cậy (bọc thẻ, thêm cảnh báo) đều là thêm văn bản vào cùng kênh với văn bản tấn công. Chúng nâng chi phí tấn công chứ không tạo ra bảo đảm.

Classifier lọc đầu vào và đầu ra chịu cùng giới hạn, vì phần lớn chúng cũng là model (suy luận của mình), lại thêm false positive: tài liệu bảo mật hay email nói về phishing đều chứa văn bản giống tấn công. Chúng đáng có để giảm tấn công đại trà và tạo tín hiệu giám sát; Anthropic cũng trình bày classifier của họ như một lớp bên cạnh huấn luyện model và red-team.

Kết luận thực hành: coi model là thành phần có thể bị điều khiển sau khi đã đọc nội dung không tin cậy, và đặt các bảo đảm ở phần code tất định bao quanh nó.

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

Nội dung không tin cậy (2) mang chỉ dẫn, model làm theo, đọc dữ liệu riêng (1), rồi gửi đi qua kênh ra ngoài (3). Bỏ một cạnh thì chuỗi đứt. Lời khuyên của ông cho người dùng cuối là tránh hẳn tổ hợp này, vì không thể trông vào guardrail.

Trifecta khó tránh vì nó hình thành do ghép tool: một tool đọc email, một tool tìm web, một tool gọi HTTP, từng cái trông vô hại, ghép lại là đủ ba cạnh. Willison chỉ ra đây là vấn đề với MCP, vì MCP khuyến khích người dùng trộn tool từ nhiều nguồn.

Meta mở rộng ý này thành "Agents Rule of Two" (bài công bố ngày 31/10/2025). Trong một phiên, agent chỉ nên có tối đa hai trong ba thuộc tính:

- [A] xử lý đầu vào không tin cậy;
- [B] truy cập hệ thống nhạy cảm hoặc dữ liệu riêng tư;
- [C] thay đổi trạng thái hoặc liên lạc ra ngoài.

Khác biệt so với trifecta nằm ở [C]: gồm cả thay đổi trạng thái, nên bao luôn trường hợp agent bị lừa xóa hay sửa thứ gì đó. Nếu cần cả ba mà không bắt đầu phiên mới, Meta nói agent không được chạy tự động và tối thiểu phải có giám sát, ví dụ phê duyệt của người. Họ cũng nói rõ quy tắc này bổ sung chứ không thay cho least privilege.

Willison phản biện một điểm: sơ đồ ban đầu ghi tổ hợp [A]+[C] là "an toàn", trong khi đầu vào không tin cậy cộng quyền thay đổi trạng thái vẫn gây hại được. Theo ông, Meta sau đó đổi nhãn thành "rủi ro thấp hơn".

## Dữ liệu đi ra ngoài bằng đường nào

Bất cứ thứ gì khiến một request rời hệ thống với nội dung do model quyết định đều là kênh rò rỉ, nên cạnh (3) rộng hơn danh sách tool gửi tin.

**Ảnh markdown.** Đường xuất hiện nhiều nhất trong các sự cố đã công bố. Giao diện chat render markdown; nếu model bị dụ xuất ra một ảnh trỏ tới máy chủ bên ngoài với dữ liệu nằm trong URL, trình duyệt của nạn nhân tự tải ảnh và dữ liệu đi theo request, không cần ai bấm gì. Cách chặn: không render ảnh từ domain ngoài, hoặc dùng Content Security Policy với allowlist domain ảnh.

**Allowlist cũng thủng.** Loạt sự cố Simon Willison ghi lại cho thấy allowlist domain chỉ tốt bằng domain yếu nhất trong danh sách:

| Sự cố (tháng theo bài ghi chép của Simon Willison) | Vì sao bộ lọc và allowlist không cứu được |
| --- | --- |
| Microsoft 365 Copilot "EchoLeak" (6/2025) | Dùng cú pháp ảnh markdown dạng reference mà bộ lọc bỏ sót, cộng một open redirect trên domain nằm trong allowlist CSP |
| GitLab Duo (5/2025) | Chỉ dẫn ẩn trong mã nguồn khiến trợ lý đọc một merge request riêng tư rồi rò nội dung qua thẻ ảnh; một lỗi hiển thị khi streaming giúp vượt bộ lọc đầu ra |
| Salesforce AgentForce, "ForcedLeak" (9/2025) | Văn bản độc nằm trong biểu mẫu khách tự điền; CSP còn liệt kê một domain đã hết hạn, nhà nghiên cứu đăng ký lại domain đó |
| Superhuman AI (1/2026) | CSP cho phép ảnh từ một domain dịch vụ biểu mẫu công cộng, nơi nhận và lưu dữ liệu gửi qua GET |

**Link và link preview.** Link có dữ liệu trong URL cần người bấm, nên yếu hơn ảnh. Nhưng nhiều ứng dụng chat tự tải URL để dựng preview (unfurl), khi đó link cũng thành kênh không cần bấm. Agent đăng tin vào kênh chat thì nên tắt unfurl.

**Tool có tác dụng phụ ra ngoài.** Gửi email, đăng comment, tạo issue ở repo công khai, gọi webhook, và cả tool "chỉ đọc" như tải một URL (URL chính là dữ liệu gửi đi). Agent chạy được code thì mọi chương trình có kết nối mạng đều là đường ra, nên chặn ở từng tool là không đủ, phải chặn ở tầng mạng.

## Rủi ro riêng của MCP

MCP (Model Context Protocol) chuẩn hóa cách nối tool vào agent; phần giao thức có trong bài [Thiết kế tool, MCP và Skill cho agent](/bai-chia-se/thiet-ke-tool-mcp-skill-cho-agent/). Ba thuật ngữ đầu dưới đây theo bài tổng hợp của Simon Willison: tool poisoning từ nghiên cứu của Invariant Labs, rug pull và tool shadowing từ bài của Elena Cross. Willison lưu ý các lỗ hổng này không nằm ở bản thân giao thức mà ở việc trao tool cho một model đang đọc nội dung không tin cậy.

**Tool poisoning.** Mô tả tool là văn bản đi thẳng vào context của model, và người dùng thường không nhìn thấy đầy đủ. Một server độc hại có thể giấu chỉ dẫn trong mô tả của một tool trông vô hại. Mô tả tool từ server bạn không kiểm soát vì thế là nội dung không tin cậy, ngang với trang web.

**Rug pull.** Server có thể đổi định nghĩa tool sau khi bạn đã duyệt. Cách phòng mình đề xuất: ghim phiên bản server, lưu hash định nghĩa tool lúc duyệt, yêu cầu duyệt lại khi hash đổi.

**Tool shadowing.** Khi nhiều server cùng nối vào một agent, một server độc hại có thể chen vào hoặc ghi đè các lời gọi vốn dành cho server tin cậy. Trong context không có ranh giới tin cậy giữa các server.

**Server chạy cục bộ.** Một server MCP cục bộ là một tiến trình chạy bằng quyền của bạn trên máy bạn; cài nó tương đương chạy mã của người lạ. Tài liệu bảo mật của MCP yêu cầu client có cài đặt một chạm phải hiển thị đầy đủ lệnh sẽ chạy và hỏi đồng ý trước, và khuyến nghị chạy server trong sandbox với quyền tối thiểu. Tài liệu đó còn có phần riêng về OAuth (confused deputy, token passthrough) cho người viết server.

## Phòng thủ theo lớp ở tầng harness

Nguyên tắc xuyên suốt: mỗi lớp giả định lớp phía trên đã thủng. Ba tầng luật tĩnh, hỏi người và sandbox đã được giới thiệu trong bài [Giải phẫu agent harness](/bai-chia-se/giai-phau-agent-harness/); phần này đi vào chi tiết liên quan tới injection.

### Sandbox filesystem và network, phải có cả hai

Bài của Anthropic về sandbox cho Claude Code nêu lý do cần đủ hai lớp cô lập: thiếu cô lập mạng, agent bị chiếm quyền có thể gửi đi các file nhạy cảm như SSH key; thiếu cô lập filesystem, agent có thể thoát khỏi sandbox rồi giành lại quyền truy cập mạng. Chi tiết triển khai đáng học:

- Cô lập ở tầng hệ điều hành (bubblewrap trên Linux, seatbelt trên macOS), nên áp dụng cho cả script và tiến trình con mà agent sinh ra. Kiểm tra ở tầng tool call không bao được những thứ đó.
- Mạng đi qua một proxy chạy ngoài sandbox. Proxy là nơi áp allowlist domain và hỏi người dùng khi có domain mới.
- Credential không nằm trong sandbox. Ở bản chạy trên web, git client trong sandbox chỉ giữ một credential phạm vi hẹp. Một proxy bên ngoài kiểm tra thao tác git (ví dụ chỉ được push lên đúng nhánh đã cấu hình) rồi mới gắn token thật trước khi chuyển tới GitHub.

Điểm cuối áp dụng được rộng hơn git: thay vì đưa API key vào biến môi trường của agent, đặt key ở proxy và để proxy gắn vào request tới đúng host. Thứ agent không cầm thì agent không làm lộ được.

### Allowlist egress

Chặn mặc định mọi kết nối ra, chỉ mở đích cần. Từ các sự cố ở trên, mình rút ra vài quy tắc khi lập allowlist:

- Liệt kê host chính xác, không wildcard theo domain cha.
- Với mỗi host, hỏi: bên ngoài có ghi được dữ liệu lên đây rồi đọc lại không? Kho lưu trữ công cộng, dịch vụ paste, biểu mẫu, webhook đều thuộc loại này. Open redirect trên host đó cũng tính.
- Phân biệt đọc với ghi nếu proxy làm được, ví dụ chỉ cho GET tới registry package. Lưu ý GET vẫn mang được dữ liệu trong URL (ca Superhuman).
- Rà định kỳ, vì domain hết hạn trong allowlist là lỗ hổng có thật (ca AgentForce).

Allowlist không chặn rò rỉ qua một host hợp lệ mà agent có quyền ghi công khai.

### Permission prompt và mệt mỏi phê duyệt

Hỏi người dùng trước mỗi hành động nghe an toàn nhất, thực tế thì không. Anthropic gọi hiện tượng này là "approval fatigue": hỏi liên tục khiến người dùng thôi chú ý vào thứ mình đang duyệt. Con số họ đưa ra từ sử dụng nội bộ: sandbox giảm 84% số lần hỏi quyền. Paper CaMeL cũng thừa nhận gánh nặng đặt lên người dùng và hiện tượng mỏi (user fatigue).

Hệ quả thiết kế, theo mình: coi lượt phê duyệt là tài nguyên khan hiếm.

- Việc nằm gọn trong sandbox thì không hỏi; chỉ hỏi ở ranh giới: ra ngoài sandbox, tới host mới, hành động không đảo ngược.
- Lời hỏi phải hiện tham số thật và đầy đủ (lệnh nguyên văn, người nhận, nội dung gửi đi), không phải bản model tóm tắt. Trong một bản demo của Invariant Labs mà Willison thuật lại, dữ liệu bị lấy nằm khuất về bên phải hộp xác nhận, vì giao diện không hiện thanh cuộn ngang.

### Tách quyền đọc và ghi

Đây là least privilege áp vào agent: token của agent hẹp hơn quyền của người dùng, đọc với ghi dùng credential khác nhau (ví dụ tài khoản DB chỉ đọc, token git chỉ cho một repo). Tách cả theo phiên, đúng tinh thần Rule of Two: phiên đọc nội dung không tin cậy thì không gắn tool ghi vào hệ thống nhạy cảm.

### Human-in-the-loop cho hành động không đảo ngược

Bảng phân loại dưới đây là đề xuất của mình, xếp tool theo khả năng hoàn tác:

| Loại hành động | Ví dụ | Xử lý |
| --- | --- | --- |
| Đọc trong phạm vi | Đọc file trong workspace, query DB chỉ đọc | Tự động |
| Ghi, hoàn tác được | Sửa file có git, tạo bản nháp, mở PR | Tự động, có log |
| Ghi, khó hoàn tác | Gửi email, đăng tin, merge, chạy migration | Người duyệt, thấy nội dung đầy đủ |
| Không hoàn tác được | Chuyển tiền, xóa dữ liệu, thu hồi quyền | Người duyệt, cân nhắc không trao tool này cho agent |

Muốn giảm số lần hỏi mà không giảm an toàn thì biến hành động khó hoàn tác thành hoàn tác được: tạo bản nháp thay vì gửi, mở PR thay vì push thẳng lên nhánh chính, ghi vào bảng chờ duyệt thay vì bảng thật.

### Một cổng kiểm soát tool tối thiểu

Đoạn dưới là ví dụ của mình cho ý "context đã nhiễm thì thu hẹp quyền", theo tinh thần Rule of Two. Nó là khung sườn, chưa đủ cho production.

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

Quyết định nằm trong code tất định, model không tự gỡ được cờ `tainted`: muốn gửi ra ngoài mà không bị hỏi thì bắt đầu phiên mới với context sạch. Đánh dấu cả phiên là cách thô; CaMeL ở phần dưới theo dõi tới từng giá trị.

## Design pattern: ràng buộc agent sau khi nó đọc nội dung bẩn

Paper "Design Patterns for Securing LLM Agents against Prompt Injections" (tháng 6/2025) khởi đi từ một nguyên tắc: khi agent đã nạp đầu vào không tin cậy, nó phải bị ràng buộc sao cho đầu vào đó không thể kích hoạt hành động gây hậu quả. Paper đưa ra sáu pattern. Cột "cái giá" là tóm lược của mình.

| Pattern | Cơ chế | Cái giá |
| --- | --- | --- |
| Action-Selector | Model chỉ làm công tắc chọn một hành động định sẵn, kết quả hành động không quay lại model | Chỉ làm được việc đã lường trước |
| Plan-Then-Execute | Chốt chuỗi tool call trước khi đọc dữ liệu không tin cậy | Injection vẫn đổi được tham số của các bước đã lên kế hoạch; khó với việc cần quyết định theo kết quả |
| LLM Map-Reduce | Mỗi mẩu dữ liệu bẩn do một instance cô lập, không tool, xử lý; đầu ra bị ràng buộc rồi mới gộp | Mất suy luận chéo giữa các mẩu; mẩu độc vẫn làm lệch kết quả của chính nó |
| Dual LLM | LLM có quyền không đọc dữ liệu bẩn, chỉ thấy tên biến; LLM cách ly đọc dữ liệu bẩn nhưng không có tool | Phức tạp, dễ sơ suất làm rò văn bản bẩn; đầu ra của LLM cách ly vẫn bị điều khiển được |
| Code-Then-Execute | Agent viết một chương trình gọi tool; chương trình chạy trên dữ liệu bẩn | Luồng điều khiển cố định, giá trị vẫn bị ảnh hưởng; đòi model viết code tin cậy |
| Context-Minimization | Gỡ prompt của người dùng khỏi context sau khi nó đã được dùng để chọn hành động | Câu trả lời cuối không phản ứng được với lời người dùng |

**CaMeL** (Google DeepMind, tháng 3/2025) vá một lỗ của Dual LLM, pattern Simon Willison đề xuất từ 2023. Trong Dual LLM, khi LLM có quyền nhờ LLM cách ly tìm địa chỉ email người nhận trong dữ liệu không tin cậy, kẻ tấn công kiểm soát dữ liệu đó sẽ kiểm soát luôn giá trị trả về, và tài liệu được gửi tới sai người dù luồng điều khiển không đổi. CaMeL xử lý thế này:

1. LLM có quyền chỉ thấy yêu cầu của người dùng và viết ra một chương trình bằng một tập con của Python.
2. Một interpreter riêng chạy chương trình đó và theo dõi mỗi giá trị bắt nguồn từ đâu. Giá trị trích từ email không tin cậy mang nhãn không tin cậy, nhãn lan theo mọi phép tính.
3. Trước mỗi tool call, interpreter kiểm tra chính sách dựa trên các nhãn đó. Ví dụ: tool gửi email chỉ chạy nếu người nhận đến từ nguồn tin cậy, nếu không thì hỏi người dùng.

Theo abstract của paper, trên benchmark AgentDojo CaMeL giải được 77% task với bảo đảm an toàn chứng minh được, so với 84% của hệ thống không phòng thủ. Ngoài 7 điểm đó, paper còn nêu các cái giá: phải có người viết và bảo trì chính sách bảo mật, người dùng chịu thêm gánh nặng phê duyệt, và prompt injection chưa được giải quyết hoàn toàn.

Chọn pattern nào? Theo mình nên đi từ cái hẹp nhất còn đáp ứng được yêu cầu: chatbot tra cứu đơn hàng hợp với Action-Selector, pipeline tóm tắt hàng loạt tài liệu ngoài hợp với Map-Reduce, còn trợ lý đa năng trên email và lịch mới cần tới thứ như CaMeL.

## Checklist threat-model cho một agent nội bộ

Giả sử bạn sắp triển khai một agent nội bộ, ví dụ agent đọc ticket hỗ trợ, tra hệ thống đơn hàng và soạn trả lời. Danh sách dưới đây do mình gom từ các phần trên.

**1. Kiểm kê ba cạnh**

- Agent đọc những nguồn nào, nguồn nào người ngoài tổ chức ghi vào được (ticket, email, trang web, trường ghi chú trong CRM, mô tả tool của MCP server bên thứ ba)?
- Agent chạm tới dữ liệu và hệ thống nào? Có thấy dữ liệu của khách A khi đang xử lý cho khách B không?
- Có những đường ra nào: tool gửi tin, tool gọi HTTP, render markdown, link preview, quyền ghi lên nơi công khai?
- Phiên nào hội đủ cả ba? Bỏ cạnh nào, hoặc đặt người duyệt ở đâu?

**2. Quyền và chuỗi cung ứng**

- Credential của agent có hẹp hơn quyền của người vận hành không? Đọc và ghi đã tách chưa?
- Secret có nằm trong môi trường agent chạy không? Chuyển được ra proxy không?
- Mỗi MCP server: ai viết, chạy bằng quyền gì, đã ghim phiên bản chưa, có phát hiện khi định nghĩa tool thay đổi không?

**3. Cô lập**

- Agent chạy code thì đã có sandbox filesystem lẫn network ở tầng hệ điều hành chưa?
- Egress mặc định chặn chưa? Mỗi host trong allowlist đã qua câu hỏi "bên ngoài ghi vào rồi đọc lại được không" chưa?
- Giao diện có render ảnh hay tự tải URL từ domain ngoài không?

**4. Phê duyệt**

- Hành động nào không đảo ngược? Mỗi cái đã có người duyệt, và người duyệt thấy tham số nguyên văn chưa?
- Mỗi phiên hỏi bao nhiêu lần? Hỏi nhiều thì chuyển bớt sang sandbox hoặc dạng bản nháp.

**5. Phát hiện và ứng phó**

- Mọi tool call có được log kèm tham số, đủ để dựng lại sự việc không?
- Thu hồi credential của agent mất bao lâu? Đã thử chưa?

## Điều còn mở, chưa chắc

- **Model có bao giờ tự kháng được không.** Số liệu của Anthropic cho thấy tỉ lệ tấn công thành công giảm khi thêm lớp bảo vệ, còn "The Attacker Moves Second" cho thấy các phòng thủ đã công bố bị tấn công thích ứng vượt qua. Willison nói ông không chia sẻ sự lạc quan của các tác giả paper rằng phòng thủ đáng tin sẽ sớm có. Thiết kế hôm nay nên giả định là chưa.
- **CaMeL và các pattern có dùng được cho agent đa năng không.** Bảo đảm của chúng đến từ việc hạn chế agent. Coding agent chạy hàng trăm bước, quyết định dựa trên thứ vừa đọc, khó khớp với Plan-Then-Execute hay Code-Then-Execute thuần túy. Theo quan sát của mình (chưa có khảo sát), các sản phẩm coding agent hiện dựa vào sandbox và phê duyệt nhiều hơn.
- **Human-in-the-loop có bền không.** Meta, đặc tả MCP và CaMeL đều dựa vào người duyệt ở điểm cuối, trong khi Anthropic và paper CaMeL thừa nhận người duyệt sẽ mỏi. Mình chưa thấy lời giải nào cho mâu thuẫn này ngoài việc hỏi ít đi.

## Nguồn đọc thêm

- [The lethal trifecta for AI agents](https://simonwillison.net/2025/Jun/16/the-lethal-trifecta/) và [loạt bài gắn thẻ exfiltration-attacks](https://simonwillison.net/tags/exfiltration-attacks/) (Simon Willison). Bài gốc của khái niệm, và hồ sơ các sự cố thật dùng cho bảng trong bài.
- [Model Context Protocol has prompt injection security problems](https://simonwillison.net/2025/Apr/9/mcp-prompt-injection/) (Simon Willison). Tool poisoning, rug pull, shadowing, kèm nguồn gốc từng thuật ngữ.
- [MCP Security Best Practices](https://modelcontextprotocol.io/specification/2025-06-18/basic/security_best_practices) (đặc tả MCP). Confused deputy trong OAuth, token passthrough, server cục bộ, với các yêu cầu MUST và SHOULD.
- [LLM01: Prompt Injection](https://genai.owasp.org/llmrisk/llm01-prompt-injection/) (OWASP). Định nghĩa chuẩn và danh sách biện pháp giảm thiểu, tiện để đối chiếu khi review.
- [Beyond permission prompts: making Claude Code more secure and autonomous](https://www.anthropic.com/engineering/claude-code-sandboxing) (Anthropic). Thiết kế sandbox hai lớp và proxy giữ credential, kèm con số 84%.
- [Mitigating the risk of prompt injections in browser use](https://www.anthropic.com/research/prompt-injection-defenses) (Anthropic, 11/2025). Số liệu tấn công thành công có và không có lớp bảo vệ, và lời thừa nhận vấn đề chưa được giải quyết.
- [Agents Rule of Two: A Practical Approach to AI Agent Security](https://ai.meta.com/blog/practical-ai-agent-security/) (Meta, 10/2025). Quy tắc hai trên ba với ví dụ cấu hình cho từng loại agent.
- [The Attacker Moves Second](https://arxiv.org/abs/2510.09023) (Nasr, Carlini và cộng sự, 2025), đọc kèm [bài điểm hai paper của Simon Willison](https://simonwillison.net/2025/Nov/2/new-prompt-injection-papers/). Lý do không nên tin tỉ lệ chặn do bên làm phòng thủ tự báo cáo, và phản biện với Rule of Two.
- [Design Patterns for Securing LLM Agents against Prompt Injections](https://arxiv.org/abs/2506.08837) (Beurer-Kellner và cộng sự, 2025), có [bài điểm paper của Simon Willison](https://simonwillison.net/2025/Jun/13/prompt-injection-design-patterns/). Sáu pattern với phân tích đánh đổi và case study.
- [Defeating Prompt Injections by Design](https://arxiv.org/abs/2503.18813) (Debenedetti và cộng sự, 2025), [bài giải thích CaMeL](https://simonwillison.net/2025/Apr/11/camel/) và [đề xuất Dual LLM gốc](https://simonwillison.net/2023/Apr/25/dual-llm-pattern/) của Simon Willison. Paper CaMeL, bản đọc dễ hơn, và pattern mà nó vá.
