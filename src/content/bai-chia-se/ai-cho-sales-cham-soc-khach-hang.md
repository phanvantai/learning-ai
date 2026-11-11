---
title: "AI cho bán hàng và chăm sóc khách hàng: 6 việc giao được và 3 việc phải giữ"
date: 2026-11-11
tags: [Bán hàng, Chăm sóc khách hàng, Quy trình làm việc]
description: Chuẩn bị cuộc gặp, cập nhật CRM, soạn báo giá, trả lời câu hỏi lặp lại, xử lý khiếu nại, luyện xử lý từ chối. Mỗi việc có câu lệnh mẫu, cách kiểm tra, và ranh giới AI không được vượt.
loai: ung-dung
---

Phần lớn thời gian của người bán hàng không nằm ở lúc nói chuyện với khách. Nó nằm ở việc tìm hiểu trước cuộc gặp, ghi chép sau cuộc gọi, soạn báo giá, và trả lời đi trả lời lại cùng một câu hỏi trên Zalo. Đây là những việc AI đỡ được nhiều. Phần còn lại, gồm quan hệ với khách, cam kết về giá và quyết định bồi thường, vẫn phải là của người.

Mình là kỹ sư phần mềm, không phải dân bán hàng. Bài này không dạy kỹ thuật chốt đơn. Nó chỉ ra chỗ nào trong quy trình có thể đưa AI vào, đưa như thế nào, và kiểm tra ra sao.

## Ba ranh giới đặt ra trước

Nói trước ba điều này vì chúng áp dụng cho mọi việc bên dưới.

**AI không được tự cam kết giá và chính sách.** Mô hình ngôn ngữ có thể viết ra một mức chiết khấu, thời hạn bảo hành hay điều kiện đổi trả nghe rất hợp lý và hoàn toàn không có thật. Với khách, tin nhắn gửi từ tài khoản của công ty là lời của công ty. Mọi con số và cam kết phải lấy từ bảng giá, chính sách đã duyệt, và có người đọc trước khi gửi.

**Không dán dữ liệu khách hàng vào công cụ chưa được công ty cho phép.** Tên, số điện thoại, địa chỉ, lịch sử mua hàng, nội dung trao đổi là dữ liệu cá nhân, và còn là tài sản của công ty. Luật Bảo vệ dữ liệu cá nhân (số 91/2025/QH15) có hiệu lực từ 01/01/2026. Hỏi quản lý xem công ty cho dùng công cụ nào. Khi chưa rõ, thay tên bằng "khách A", "công ty B" và xóa số điện thoại, số hợp đồng trước khi dán.

**Tin nhắn hàng loạt cá nhân hóa giả tạo phản tác dụng.** Phần này có một mục riêng ở cuối bài.

## Việc nào giao, việc nào giữ

| Giao cho AI (có kiểm tra) | Giữ lại cho người |
| --- | --- |
| Tóm tắt thông tin công khai về khách | Xác minh thông tin, chọn góc tiếp cận |
| Soạn câu hỏi khai thác nhu cầu | Hỏi, nghe, và đọc phản ứng của khách |
| Chuyển ghi chú thành cập nhật CRM, email follow-up | Xác nhận điều đã hứa với khách |
| Viết phần lời của báo giá, đề xuất | Giá, chiết khấu, điều khoản thanh toán |
| Soạn và chuẩn hóa bộ câu trả lời mẫu | Duyệt nội dung chính sách trong câu trả lời |
| Soạn nháp thư trả lời khiếu nại | Quyết định bồi thường, đổi trả, ngoại lệ |
| Đóng vai khách để bạn luyện tập | Cuộc nói chuyện thật |

## 1. Chuẩn bị trước cuộc gặp

Với khách doanh nghiệp, việc cần làm là đọc website, tin tức, trang tuyển dụng của họ để hiểu họ đang làm gì và có thể cần gì. AI làm việc này nhanh, với một điều kiện: công cụ phải có chức năng tìm kiếm web và phải đưa link nguồn. Nếu hỏi một công cụ không tra web về một công ty Việt Nam cỡ vừa, rất dễ nhận về thông tin bịa hoặc nhầm sang công ty trùng tên.

```text
Tôi sắp gặp [chức danh] của [tên công ty, website] để giới thiệu [sản phẩm/dịch vụ của tôi].
Hãy tìm thông tin công khai và tóm tắt trong một trang:
1. Công ty làm gì, quy mô, khách hàng chính (kèm link nguồn cho từng ý).
2. Tin tức đáng chú ý trong 12 tháng gần đây.
3. Dấu hiệu cho thấy họ có thể cần sản phẩm của tôi (ví dụ tin tuyển dụng, mở chi nhánh).
4. Mười câu hỏi mở để khai thác nhu cầu, xếp từ hỏi về hiện trạng đến hỏi về hệ quả
   nếu vấn đề không được giải quyết.
Ý nào không tìm được nguồn thì ghi "không tìm thấy", đừng suy đoán.
```

Cách kiểm tra: bấm vào link của những ý bạn định nhắc trong cuộc gặp. Nói sai về công ty của khách ngay trước mặt họ thì tệ hơn là không chuẩn bị. Kiểm tra cả ngày tháng của tin, vì AI hay trộn tin cũ với tin mới.

Với khách cá nhân, đừng dùng AI để "điều tra" trang cá nhân của họ. Dùng ghi chú từ các lần trao đổi trước là đủ, và khách cũng không thấy bị soi.

## 2. Từ ghi chú cuộc gọi thành cập nhật CRM và email follow-up

Sau cuộc gọi, ghi chú thường là vài dòng gõ vội. Việc biến chúng thành bản ghi gọn trong CRM (hệ thống quản lý thông tin khách hàng, nhiều nơi chỉ là một file Google Sheets) và một email xác nhận hay bị hoãn đến cuối ngày, khi bạn đã quên một nửa. Giao việc này cho AI ngay sau cuộc gọi.

```text
Dưới đây là ghi chú thô của tôi sau cuộc gọi với khách (đã thay tên thật bằng "khách A").
Hãy viết hai phần:

PHẦN 1 - Cập nhật CRM, đúng các mục:
- Nhu cầu chính của khách
- Băn khoăn hoặc lý do chưa quyết
- Người ra quyết định và người ảnh hưởng
- Việc tôi đã hứa (ai làm, hạn nào)
- Bước tiếp theo và ngày hẹn
Mục nào ghi chú không nói thì ghi "chưa rõ".

PHẦN 2 - Email follow-up dưới 150 chữ, giọng lịch sự, xưng "em", gọi khách là "anh":
cảm ơn, nhắc lại 2-3 điểm đã thống nhất, nêu việc tôi sẽ gửi và hạn, đề xuất bước tiếp theo.
Chỉ dùng thông tin trong ghi chú. Không thêm giá, ưu đãi hay cam kết nào tôi không ghi.

[Dán ghi chú]
```

Cách kiểm tra: đọc mục "việc tôi đã hứa" trước tiên. AI hay làm hai lỗi ở đây: biến "để em hỏi lại sếp về mức giảm" thành "em sẽ gửi mức giảm", và tự điền một ngày hẹn cho tròn. Mỗi lời hứa trong email là một việc bạn phải làm thật.

Nếu công ty dùng công cụ ghi âm và tự ghi biên bản cuộc gọi, hai điều cần làm trước: công cụ đó phải được công ty duyệt, và khách phải biết cuộc gọi được ghi lại.

## 3. Soạn báo giá và đề xuất từ mẫu

Một bản đề xuất có hai phần khác hẳn nhau. Phần lời (hiểu nhu cầu khách thế nào, giải pháp là gì, vì sao phù hợp) là phần AI viết tốt nếu có ghi chú cuộc gặp. Phần số (đơn giá, số lượng, chiết khấu, thuế GTGT, tổng tiền, điều khoản thanh toán) thì bạn tự điền từ bảng giá và để Excel tính.

```text
Đây là mẫu đề xuất của công ty tôi và ghi chú nhu cầu của khách A.
Hãy viết lại ba mục "Hiểu về nhu cầu của quý công ty", "Giải pháp đề xuất"
và "Lý do phù hợp" cho đúng trường hợp của khách này.
- Dùng lại đúng từ ngữ khách đã dùng khi mô tả vấn đề của họ.
- Mỗi lợi ích phải gắn với một nhu cầu khách đã nói, không liệt kê tính năng chung.
- Mọi chỗ cần con số (giá, số lượng, thời gian triển khai, bảo hành) để nguyên
  dạng "[ĐIỀN: ...]" cho tôi tự điền.
Không viết thêm cam kết nào ngoài những gì có trong mẫu.

[Mẫu đề xuất]
[Ghi chú nhu cầu]
```

Cách kiểm tra: tìm (Ctrl+F) chữ "[ĐIỀN" để chắc không sót ô nào. Sau đó đọc lại phần lời xem có câu nào hứa thứ sản phẩm không làm được. AI viết đề xuất có xu hướng nói quá cho thuyết phục, kiểu "đáp ứng hoàn toàn mọi yêu cầu". Dò cả tên công ty, tên người nhận: lỗi kinh điển của việc làm từ mẫu là còn sót tên khách cũ.

## 4. Bộ câu trả lời mẫu cho câu hỏi lặp lại trên Zalo, Messenger

Phí ship bao nhiêu, có xuất hóa đơn không, bảo hành thế nào, bao lâu nhận hàng. Mười câu hỏi quen thuộc thường chiếm phần lớn tin nhắn. Khi mỗi nhân viên trả lời một kiểu, khách nhận được thông tin lệch nhau và giọng mỗi lúc một khác.

Cách làm gồm ba bước:

1. Gom câu hỏi thật. Chép các câu khách hay hỏi từ lịch sử chat (bỏ tên, số điện thoại) và nhờ AI gom thành nhóm.
2. Viết câu trả lời chuẩn. Nội dung lấy từ chính sách đã duyệt, AI chỉnh giọng.
3. Lưu thành bộ trả lời nhanh. Cả Zalo lẫn công cụ quản lý trang Facebook đều có chức năng lưu câu trả lời soạn sẵn; tên gọi và vị trí thay đổi theo phiên bản nên bạn tìm trong phần cài đặt tin nhắn.

```text
Đây là chính sách của cửa hàng (giao hàng, đổi trả, bảo hành, thanh toán)
và danh sách 15 câu khách hay hỏi.
Với mỗi câu hỏi, viết một câu trả lời mẫu:
- Tối đa 3 câu, đọc vừa một màn hình điện thoại.
- Xưng "em", gọi khách là "anh/chị", lịch sự, không dùng biểu tượng cảm xúc.
- Trả lời thẳng vào câu hỏi ở câu đầu tiên.
- Chỉ dùng thông tin có trong chính sách. Câu hỏi nào chính sách không nói đến
  thì ghi "CHƯA CÓ CHÍNH SÁCH - cần hỏi quản lý" thay vì tự trả lời.
- Chỗ nào phụ thuộc từng đơn (phí ship theo khu vực, tồn kho) để dạng [điền].

[Chính sách]
[Danh sách câu hỏi]
```

Các dòng "CHƯA CÓ CHÍNH SÁCH" là kết quả có ích: chúng chỉ ra chỗ nhân viên đang phải tự ứng biến. Bộ câu trả lời cần một người duyệt và cần xem lại mỗi khi chính sách đổi, vì câu mẫu sai sẽ được gửi đi rất nhiều lần.

## 5. Xử lý khiếu nại

Khi khách đang bực, câu trả lời đầu tiên quyết định nhiều thứ. AI giúp được ở việc soạn một bản nháp bình tĩnh, nhận đúng vấn đề, không đổ lỗi, đặc biệt lúc chính bạn cũng đang khó chịu. Quyết định xử lý (hoàn tiền, đổi hàng, bồi thường bao nhiêu, có nhận lỗi hay không) phải do người có thẩm quyền đưa ra trước, rồi mới đưa cho AI viết.

```text
Khách gửi khiếu nại dưới đây (đã bỏ tên). Hướng xử lý đã được quản lý duyệt:
[ví dụ: đổi sản phẩm mới, cửa hàng chịu phí ship hai chiều, không hoàn tiền].
Hãy soạn tin nhắn trả lời:
1. Câu đầu nhắc lại đúng vấn đề khách gặp, bằng lời của khách.
2. Xin lỗi về trải nghiệm của khách, không viện lý do, không đổ cho bên vận chuyển.
3. Nêu rõ cửa hàng sẽ làm gì, khi nào, khách cần làm gì.
4. Dưới 120 chữ, xưng "em", không dùng câu khuôn mẫu kiểu
   "chúng tôi rất tiếc vì sự bất tiện này".
Không thêm bất kỳ khoản bồi thường, ưu đãi hay cam kết nào ngoài hướng xử lý ở trên.

[Tin nhắn của khách]
```

Cách kiểm tra: so từng cam kết trong bản nháp với hướng xử lý đã duyệt. AI rất hay "tặng thêm" một mã giảm giá cho lần mua sau để xoa dịu. Với khiếu nại có yếu tố sức khỏe, an toàn, hoặc khách nhắc đến cơ quan chức năng, báo chí, luật sư: chuyển cho quản lý, đừng trả lời bằng bản nháp AI. Doanh nghiệp có nghĩa vụ theo pháp luật về bảo vệ quyền lợi người tiêu dùng, và lời nhận lỗi bằng văn bản có hệ quả.

## 6. Luyện xử lý từ chối bằng nhập vai

Việc này không tạo ra tài liệu nào, nhưng là cách dùng AI ít rủi ro nhất trong bài vì không có khách thật nào bị ảnh hưởng. Bạn nhờ AI đóng vai một khách khó tính và tập trả lời những câu như "giá cao quá", "để anh suy nghĩ thêm", "bên kia báo rẻ hơn".

```text
Hãy đóng vai trưởng phòng mua hàng của một công ty sản xuất 200 nhân viên.
Bạn đang cân nhắc [sản phẩm của tôi] nhưng thấy giá cao hơn nhà cung cấp hiện tại 15%
và ngại đổi vì sợ gián đoạn. Bạn bận, nói ngắn, không dễ bị thuyết phục.
Tôi là nhân viên bán hàng. Luật chơi:
- Mỗi lượt bạn chỉ nói 1-3 câu như người thật, rồi chờ tôi trả lời.
- Không tự nhượng bộ chỉ vì tôi lịch sự; chỉ đổi ý khi tôi nêu được lý do hợp lý.
- Khi tôi gõ "DỪNG", thoát vai và nhận xét: câu nào của tôi thuyết phục,
  câu nào làm bạn mất thiện cảm, tôi đã bỏ lỡ câu hỏi nào đáng lẽ phải hỏi.
Bắt đầu bằng câu: "Bên em báo giá cao hơn chỗ anh đang dùng khá nhiều đấy."
```

Giới hạn cần biết: AI đóng vai thường dễ tính hơn khách thật, và phần nhận xét của nó hay nghiêng về khen. Dòng "không tự nhượng bộ" giúp được một phần. Coi đây là chỗ tập cho trôi miệng và thử nhiều cách diễn đạt; phản hồi đáng tin hơn vẫn là từ đồng nghiệp có kinh nghiệm và từ khách thật.

## Tin nhắn hàng loạt cá nhân hóa giả tạo

AI khiến việc viết một nghìn tin nhắn "riêng cho từng người" trở nên dễ. Kết quả thường là kiểu: "Chào anh Hùng, em rất ấn tượng với hành trình phát triển của công ty anh trong lĩnh vực logistics." Người nhận nhận ra ngay vì ba lý do: họ đã nhận nhiều tin y hệt, lời khen không gắn với việc cụ thể nào, và nếu AI lấy sai một chi tiết (nhầm chức danh, nhầm ngành) thì tin nhắn thành bằng chứng rằng bạn không hề tìm hiểu.

Cá nhân hóa có tác dụng khi nó dựa trên điều bạn thật sự biết: khách đã mua gì, lần trước hỏi gì, đang vướng gì. Vài nguyên tắc:

- Gửi ít người hơn, mỗi tin dựa trên một dữ kiện có thật trong CRM. AI soạn nháp từ dữ kiện đó, bạn đọc từng tin trước khi gửi.
- Nếu là tin thông báo chung (lịch nghỉ lễ, chương trình mới), cứ viết như thông báo chung. Một thông báo thẳng thắn dễ chịu hơn một lời thân mật giả.
- Chỉ gửi cho người đã đồng ý nhận tin. Tin quảng cáo gửi cho người chưa đồng ý còn vướng quy định về chống tin nhắn rác và bảo vệ dữ liệu cá nhân; hãy đối chiếu văn bản hiện hành và quy định của nền tảng nhắn tin bạn dùng.

## Nếu chỉ bắt đầu với một việc

Chọn việc 2. Ngay sau cuộc gọi tới, dán ghi chú thô (đã thay tên khách) và nhờ AI viết cập nhật CRM cùng email follow-up, rồi tự dò mục "việc tôi đã hứa". Bạn sẽ thấy ngay AI đỡ được phần gõ chữ, còn phần nhớ đúng mình đã hứa gì với khách vẫn là việc của bạn.

## Nguồn đọc thêm

- [Những điểm mới trong Luật Bảo vệ dữ liệu cá nhân từ 1/1/2026 (VnEconomy)](https://vneconomy.vn/nhung-diem-moi-trong-luat-bao-ve-du-lieu-ca-nhan-tu-112026.htm): bối cảnh pháp lý cho ranh giới "không dán dữ liệu khách hàng tùy tiện". Đây là bài tổng hợp; khi cần chắc chắn hãy đọc toàn văn luật và hỏi bộ phận pháp lý của công ty.
