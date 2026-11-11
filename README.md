# Learning AI

Website chia sẻ, mentoring và đào tạo ứng dụng AI cho công việc hằng ngày.

- Spec sản phẩm: `docs/specs/`
- Thuật ngữ: `CONTEXT.md`
- Trợ lý AI cá nhân (app desktop + dòng lệnh, Rust), tách riêng khỏi website: `harness/` (xem `harness/README.md`)

## Phát triển

Cần Node.js 24 (xem `.node-version`; Astro yêu cầu tối thiểu 22.23.4).

```sh
npm install
npm run dev      # chạy local
npm test         # chạy test
npm run check    # kiểm tra kiểu
npm run build    # build ra dist/
```

Deploy: Vercel tự build khi merge vào `main`; mỗi PR có một bản preview.

Chỗ còn `TODO(owner)` là nội dung thật người sáng lập cần tự điền (`src/site.ts`, trang Giới thiệu, `src/contact-form.ts`).

### Bật form liên hệ trang Doanh nghiệp (`TODO(owner)`)

Web tĩnh không có backend, nên form liên hệ trên `/doanh-nghiep/` gửi qua [Formspree](https://formspree.io) (gói miễn phí có giới hạn số lần gửi mỗi tháng, đủ cho giai đoạn thí điểm; xem hạn mức hiện tại trên trang giá của Formspree). Khi chưa cấu hình, form bị khóa và hướng khách sang kênh chat, không ai gửi vào hư không.

1. Đăng ký Formspree bằng email muốn nhận liên hệ (email người sáng lập) và xác nhận email.
2. Tạo form mới (tên gợi ý: "Learning AI – Doanh nghiệp"), copy địa chỉ dạng `https://formspree.io/f/abcdwxyz`.
3. Dán địa chỉ đó vào `BUSINESS_CONTACT_FORM_ENDPOINT` trong `src/contact-form.ts` (chỗ duy nhất cần sửa), mở PR rồi merge.
4. Trên bản preview của PR, gửi thử một lần và kiểm tra hộp thư (cả thư mục spam).

Có JavaScript thì khách thấy thông báo thành công/lỗi bằng tiếng Việt ngay trên trang; tắt JavaScript thì form vẫn gửi được, nhưng khách sẽ thấy trang của Formspree (bằng tiếng Anh, có thể kèm bước xác minh chống spam) thay vì thông báo trên trang.

## Viết Bài chia sẻ mới

1. Tạo file Markdown trong `src/content/bai-chia-se/`. Tên file là đường dẫn của bài, viết không dấu, nối
   bằng gạch ngang: `dung-ai-viet-bien-ban-hop.md` → `/bai-chia-se/dung-ai-viet-bien-ban-hop/`.
2. Đầu file là frontmatter, đủ 5 mục:

   ```md
   ---
   title: Dùng AI viết biên bản họp trong 5 phút
   date: 2026-11-11
   tags: [Họp, Prompt mẫu]
   description: Một hai câu tóm tắt bài, hiện ở danh sách và Trang chủ.
   loai: ung-dung
   ---

   Nội dung bài viết bằng Markdown: ## tiêu đề mục, danh sách, **in đậm**, khối code...
   ```

   - `date`: ngày đăng, dạng `YYYY-MM-DD`. Danh sách xếp bài mới nhất lên đầu; Trang chủ hiện 3 bài mới nhất.
   - `loai`: Loại bài. `ung-dung` (mặc định nếu bỏ trống) cho bài dùng AI trong công việc hằng ngày;
     `chuyen-sau` cho bài kỹ thuật về AI, agent, harness. Trang danh sách và Trang chủ tách riêng hai loại.
   - `tags`: ít nhất một thẻ chủ đề (chủ đề của bài, ví dụ "Email", "Excel"; không phải đối tượng đọc),
     viết có dấu như muốn hiển thị. Mỗi thẻ tự có trang lọc
     (`Prompt mẫu` → `/bai-chia-se/chu-de/prompt-mau/`). Dùng lại đúng tên thẻ cũ để bài vào chung một nhóm.
3. Chạy `npm run dev` để xem thử, rồi `npm test`: test báo lỗi nếu frontmatter thiếu mục, hoặc bài dùng
   từ trong danh sách cần tránh của `tests/glossary.test.ts` (lấy từ `CONTEXT.md`, ví dụ "khóa học") hay
   tự xưng "chuyên gia AI" (`docs/specs/positioning.md`). Test không thay được việc tự đọc lại bài.
4. Commit và mở PR; merge vào `main` là bài lên web.

Bài mẫu: `src/content/bai-chia-se/tom-tat-email-dai-bang-ai.md`.

Trong bài, khối code ` ```text ` dùng cho câu lệnh mẫu (trang tự thêm nút "Sao chép"); tiêu đề `##` tự vào mục lục bên trái bài.

## Giao diện

Ý tưởng: cuốn vở ghi của người đang học. Nền sáng là trang vở kẻ ô ly (mực xanh, bút dạ quang vàng, bút đỏ);
nền tối là bảng đen. Không gradient, không đổ bóng, không bo góc: thứ bậc đến từ chữ, đường kẻ và khoảng trắng.

- Token màu và chữ: đầu file `src/styles/global.css`. Component không viết mã màu trực tiếp.
- Chữ: Source Serif 4 (tiêu đề, nội dung bài), Be Vietnam Pro (giao diện), JetBrains Mono (nhãn, code).
- Khối dựng trang: `.page-head` (đầu trang), `.sheet` (một mục: tiêu đề bên trái, nội dung bên phải),
  `.facts`, `.rows`, `.ticks`, `.table`.

## Chương trình Mentoring

Nội dung 4 buổi, ba Hướng, phiếu khởi động và tiêu chí nằm ở `src/mentoring-program.ts` (trang Mentoring và
Trang chủ cùng đọc); thiết kế và lý do ở `docs/specs/mentoring-program.md`. Sửa chương trình thì sửa cả hai.
