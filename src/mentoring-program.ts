/**
 * Chương trình Mentoring 4 buổi (`docs/specs/mentoring-program.md`). Trang Mentoring và Trang chủ cùng
 * đọc từ đây; sửa chương trình thì sửa cả spec.
 */

/** Một hướng của Mentoring, chọn theo việc của học viên. */
export interface Huong {
  ten: string;
  laGi: string;
  hopKhi: string;
  can: string;
  viDu: string;
}

export const HUONG: readonly Huong[] = [
  {
    ten: 'Skill',
    laGi: 'Một trợ lý chuyên việc: hướng dẫn cố định, tài liệu mẫu và câu lệnh, chạy trong không gian dự án của công cụ chat.',
    hopKhi: 'Việc soạn, tóm tắt, rà soát, trả lời theo mẫu.',
    can: 'Một tài khoản công cụ chat AI.',
    viDu: 'Trợ lý viết mô tả công việc theo mẫu công ty; trợ lý trả lời khách theo chính sách.',
  },
  {
    ten: 'Automation',
    laGi: 'Một luồng tự chạy nối các ứng dụng: có đầu vào mới thì AI xử lý rồi ghi hoặc gửi kết quả.',
    hopKhi: 'Việc lặp theo lịch hoặc theo sự kiện, đầu vào đến từ form, email, bảng tính.',
    can: 'Tài khoản công cụ tự động hóa (mình gợi ý theo nhu cầu).',
    viDu: 'Phản hồi khách từ form, phân loại, ghi vào bảng, báo cho nhóm.',
  },
  {
    ten: 'Tool',
    laGi: 'Một chương trình nhỏ chạy trên máy bạn, mình viết cùng bạn bằng coding agent.',
    hopKhi: 'Xử lý file, đối chiếu dữ liệu, việc cần tính chính xác.',
    can: 'Máy tính cài được phần mềm. Phần code mình lo.',
    viDu: 'Đối chiếu sao kê với sổ; gộp và soát nhiều file Excel.',
  },
];

/** Một khoảng thời gian trong buổi, tính bằng phút từ đầu buổi. */
export interface HoatDong {
  phut: string;
  noiDung: string;
}

export interface Buoi {
  so: number;
  tuan: string;
  ten: string;
  mucTieu: string;
  hoatDong: readonly HoatDong[];
  sanPham: string;
  /** Việc học viên làm trước buổi kế tiếp; buổi cuối không có. */
  baiTap?: string;
}

export const BUOI: readonly Buoi[] = [
  {
    so: 1,
    tuan: 'Tuần 1',
    ten: 'Khảo sát và chốt đề bài',
    mucTieu: 'Hai bên hiểu giống nhau về việc cần làm và thế nào là “xong”.',
    hoatDong: [
      { phut: '0–15', noiDung: 'Bạn chia sẻ màn hình, làm việc đó một lượt như thường ngày. Mình quan sát và hỏi.' },
      { phut: '15–30', noiDung: 'Cùng vẽ lại quy trình thành các bước; đánh dấu bước tốn thời gian và bước cần phán đoán của người.' },
      { phut: '30–50', noiDung: 'Viết đề bài một trang: đầu vào, đầu ra, tiêu chí đạt, việc AI không được làm, ranh giới dữ liệu. Chọn hướng.' },
      { phut: '50–60', noiDung: 'Dựng bản thô đầu tiên ngay trong buổi để bạn thấy hướng đi.' },
    ],
    sanPham: 'Đề bài một trang và số đo gốc: việc này đang tốn bao nhiêu phút mỗi lần.',
    baiTap: 'Gom 5–10 đầu vào thật kèm kết quả mong muốn (bộ mẫu kiểm tra), ẩn danh nếu cần. Bấm giờ 2 lần làm tay.',
  },
  {
    so: 2,
    tuan: 'Tuần 2',
    ten: 'Dựng bản chạy được',
    mucTieu: 'Bạn tự chạy được phiên bản 1 trên tài khoản hoặc máy của mình.',
    hoatDong: [
      { phut: '0–25', noiDung: 'Chạy bản thô trên bộ mẫu. Đọc từng kết quả, ghi lỗi và gom lỗi thành nhóm.' },
      { phut: '25–50', noiDung: 'Sửa theo nhóm lỗi lớn nhất: viết lại hướng dẫn, thêm tài liệu mẫu, tách bước, nối luồng hoặc viết code tùy hướng.' },
      { phut: '50–60', noiDung: 'Bạn tự chạy lại từ đầu, không có mình cầm tay. Chốt cách dùng hằng ngày.' },
    ],
    sanPham: 'Phiên bản 1 chạy được và bảng nhóm lỗi.',
    baiTap: 'Dùng phiên bản 1 cho việc thật ít nhất 3 lần, ghi nhật ký sử dụng: ngày, đầu vào, đạt hay chưa, phải sửa tay gì, mất bao lâu.',
  },
  {
    so: 3,
    tuan: 'Tuần 3',
    ten: 'Sửa theo việc thật',
    mucTieu: 'Xử lý những gì chỉ lộ ra khi dùng thật.',
    hoatDong: [
      { phut: '0–15', noiDung: 'Soát nhật ký sử dụng, chọn 2–3 vấn đề đáng sửa nhất.' },
      { phut: '15–45', noiDung: 'Sửa, rồi chạy lại trên bộ mẫu để chắc không làm hỏng cái đang đúng. Thêm trường hợp mới vào bộ mẫu.' },
      { phut: '45–60', noiDung: 'Thêm bước tự kiểm tra và cách xử lý khi đầu vào lạ. Viết danh sách kiểm tra trước khi dùng kết quả.' },
    ],
    sanPham: 'Phiên bản 2 và danh sách kiểm tra trước khi dùng kết quả.',
    baiTap: 'Dùng phiên bản 2 cả tuần, cố không hỏi mình. Ghi lại chỗ kẹt và chỗ muốn đổi.',
  },
  {
    so: 4,
    tuan: 'Tuần 4',
    ten: 'Bàn giao và tự chủ',
    mucTieu: 'Bạn tự bảo trì được, và biết làm việc tiếp theo một mình.',
    hoatDong: [
      { phut: '0–20', noiDung: 'Bạn tự làm một thay đổi nhỏ từ danh sách “muốn đổi”. Mình quan sát, chỉ nhắc khi kẹt.' },
      { phut: '20–35', noiDung: 'Viết trang hướng dẫn sử dụng và bảo trì: dùng thế nào, khi sai thì xem ở đâu, khi nào cần sửa.' },
      { phut: '35–45', noiDung: 'Đo lại thời gian, so với số đo gốc. Ghi nhận cả phần chưa đạt.' },
      { phut: '45–60', noiDung: 'Chọn việc tiếp theo trong danh sách công việc của bạn, phác kế hoạch tự làm. Trao đổi về case study.' },
    ],
    sanPham: 'Phiên bản cuối, trang hướng dẫn, số đo trước và sau.',
  },
];

/** Câu hỏi trong phiếu khởi động, học viên trả lời qua chat trước khi vào gói. */
export const PHIEU_KHOI_DONG: readonly string[] = [
  'Bạn làm công việc gì, hằng tuần lặp lại những việc nào?',
  'Việc nào bạn muốn AI làm giúp nhất? Hiện mỗi lần mất bao lâu, mỗi tuần mấy lần?',
  'Đầu vào của việc đó là gì (file, email, tin nhắn, form…) và đầu ra phải trông thế nào?',
  'Làm sao biết kết quả đúng? Ai kiểm tra?',
  'Dữ liệu có gì nhạy cảm không? Công ty cho phép dùng công cụ AI nào?',
  'Bạn đang dùng công cụ AI nào, có tài khoản trả phí không?',
];

/** Bốn tiêu chí để một việc được nhận vào Mentoring; thiếu một thì thu hẹp hoặc đổi việc. */
export const TIEU_CHI_CHON_VIEC: readonly { ten: string; cauHoi: string }[] = [
  { ten: 'Lặp lại', cauHoi: 'Bạn có làm việc này ít nhất mỗi tuần một lần không?' },
  { ten: 'Đầu vào rõ', cauHoi: 'Đầu vào có dạng ổn định, lấy được mẫu thật không?' },
  { ten: 'Kiểm tra được', cauHoi: 'Bạn có tự nhìn ra kết quả đúng hay sai trong vài phút không?' },
  { ten: 'Sai không gây hại lớn', cauHoi: 'Nếu AI sai mà lọt thì sửa lại được không? Có người duyệt trước khi ra ngoài không?' },
];

export const TIEU_CHI_HOAN_THANH: readonly string[] = [
  'Bạn tự chạy được sản phẩm mà không cần mình.',
  'Đã dùng cho việc thật ít nhất 5 lần.',
  'Có số đo trước và sau, kể cả khi kết quả không như kỳ vọng.',
  'Có trang hướng dẫn sử dụng và bảo trì.',
];

export const HOP: readonly string[] = [
  'Bạn có một việc lặp lại cụ thể muốn giải.',
  'Bạn sẵn sàng dùng thật giữa các buổi, khoảng 1–2 giờ mỗi tuần.',
  'Bạn được phép dùng công cụ AI với dữ liệu đó, hoặc ẩn danh được.',
];

export const CHUA_HOP: readonly string[] = [
  'Bạn muốn học AI tổng quát: Bài chia sẻ hợp hơn và không mất phí.',
  'Việc chỉ làm một lần rồi thôi.',
  'Dữ liệu bị cấm đưa ra ngoài và không ẩn danh được.',
  'Bạn cần một phần mềm lớn cho nhiều người dùng: ngoài phạm vi 4 buổi.',
];
