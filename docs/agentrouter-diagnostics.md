# Kiểm tra AgentRouter

## Cập nhật 30/10/2026: chat bị từ chối theo yêu cầu

Đối chiếu transcript OpenClaw và log upstream cho đúng hai lượt chat:

| Lượt | Kết quả upstream | Bằng chứng |
| --- | --- | --- |
| `Xin chào`, 11:30 | HTTP 200 | Model trả `Chào bạn! Tôi có thể giúp gì cho bạn hôm nay?`; ghi nhận 4.775 token đầu vào, 166 token đầu ra (gồm reasoning) |
| Hỏi số thiết bị đang hoạt động, 11:30 | HTTP 400 `content-blocked` | Bị từ chối sau 258 ms; transcript ghi 0 token; chưa có lời gọi công cụ |

Cả hai lượt đều đăng ký cùng nhóm 7 công cụ client. Lượt lỗi chưa đến
bước thực thi công cụ, nên chưa có bằng chứng cho rằng API thiết bị hoặc
schema công cụ gây lỗi này. Không thấy timeout hay context overflow trong
các lượt được kiểm tra.

Kiểm tra bổ sung qua chính REST chat HeraMind, dùng backend đang hoạt động
`openai_6e8fc8c5`, phiên tạm với ghi nhớ tắt và cấu hình prompt/công cụ mặc
định. Không thêm tiền tố, dịch hoặc sửa nội dung để vượt bộ lọc. Các phiên
tạm được xóa sau kiểm tra:

| Điều kiện | Kết quả | Thời gian toàn lượt |
| --- | --- | --- |
| Phiên mới, `Xin chào` | `content-blocked`, không gọi công cụ | 643 ms |
| Hỏi số thiết bị sau lượt chào trên | `content-blocked`, không gọi công cụ | 587 ms |
| Phiên mới, hỏi số thiết bị | `content-blocked`, không gọi công cụ | 525 ms |
| Phiên mới, `Reply with only the word OK.` | Trả `OK` | 3.131 ms |

`GET /api/devices` hoạt động, hiện trả danh sách rỗng. Đây là kết quả API
trực tiếp; lượt chat bị chặn không thực hiện được truy vấn thiết bị.

Kết luận giới hạn: có khác biệt giữa các yêu cầu tiếng Việt và yêu cầu
tiếng Anh trong lần kiểm tra này, nhưng lời chào tiếng Việt trước đó đã
thành công. Chưa đủ bằng chứng để khẳng định mọi nội dung tiếng Việt bị
chặn hoặc xác định quy tắc cụ thể của AgentRouter. Cần nhà cung cấp tra
request ID và xác nhận hợp đồng hỗ trợ nội dung/ngôn ngữ/ngữ cảnh.

Request ID tiêu biểu cho câu hỏi thiết bị:
`20261030123945449642415rwkn9ajSiyVB`.

HeraMind đã sửa hai lỗi cục bộ làm việc chẩn đoán sai lệch:

- Nhận và hiển thị lỗi nằm trong SSE HTTP 200, thay vì coi đó là câu trả
  lời rỗng rồi tự thử lại. Stream lỗi kết thúc sau khi báo lỗi.
- Khi stream thực sự rỗng và được phục hồi, dùng câu hỏi hiện tại thay vì
  câu hỏi đầu tiên trong lịch sử. Transcript cũ cho thấy lượt hỏi thiết bị
  đã bị nhắc lại thành `User: Xin chào` ở nhánh phục hồi.

Kiểm thử hồi quy: `cargo test -p heramind-agent --test openai_stream_errors`.
Các kết quả ngày 29/10 dưới đây là lịch sử kiểm chứng, không bảo đảm mọi
lượt chat hiện tại đều được AgentRouter chấp nhận.

## Kiểm chứng ngày 29/10/2026

### Kết quả cập nhật: đã kết nối thành công

Sau khi xác nhận khóa chỉ được cấp cho `gpt-6.1-sol`, đã dùng client
OpenClaw 2026.10.8 thật và nhận HTTP 200 từ AgentRouter `/v1/responses`,
model trả lời `OK`. HeraMind hiện kết nối qua gateway loopback
`http://127.0.0.1:19376/v1`, model gateway `openclaw/default`.

Đã kiểm chứng từ gateway: phản hồi thường, SSE, lời gọi hàm có cấu trúc và
kết quả hàm gửi lại. Bộ kiểm tra HeraMind cũng trả `success: true`,
`response: OK` (khoảng 3,8 giây ở lần kiểm tra đầu).

Các phép thử trước đó trong báo cáo dưới đây phản ánh kết nối trực tiếp,
không phải trạng thái hiện tại của gateway. Qua OpenClaw, lỗi sâu hơn được
phân biệt: `gpt-6-astra` yêu cầu Responses API (400 khi dùng Chat Completions),
sau đó bị từ chối quyền model (403). `deepseek-v4-flash` và
`claude-opus-4-8` cũng bị từ chối quyền. Việc xác định đúng model được cấp
là cần thiết bên cạnh lựa chọn client và giao thức.

Xem [cách chạy gateway](agentrouter.md#kết-nối-đang-hoạt-động-qua-openclaw).

### Cấu hình và kết quả thực tế

HeraMind 0.9.24 đang sử dụng giao thức OpenAI, endpoint
`https://agentrouter.org/v1`, model `gpt-6-astra`. Backend xác nhận có API key
đã lưu; trạng thái này không đồng nghĩa khóa đã được nhà cung cấp xác thực.

Phép kiểm tra phản hồi bằng chính backend đã lưu nhận HTTP 401 với
`type: unauthorized_client_error`, thông báo `unauthorized client detected`.
Trước đó cùng backend dùng model `gpt-6.1-sol` cũng gặp lỗi này.

Các phép thử độc lập bổ sung không sử dụng khóa thật:

| Phép thử | Kết quả |
| --- | --- |
| `GET /v1/models`, không gửi khóa | 401 `unauthorized_client_error` |
| `GET /v1/models`, Bearer token giả dành cho chẩn đoán | Cùng lỗi 401 |
| `GET /v1/models`, token giả và `User-Agent: HeraMind/0.9.24` | Cùng lỗi 401 |
| `POST /v1/chat/completions`, token giả | Cùng lỗi 401 |
| `POST /v1/messages`, `x-api-key` giả và `anthropic-version` | Cùng lỗi 401 |
| `GET https://co.agentrouter.org/v1/models`, không gửi thông tin xác thực | HTTP 401 với `msg: Missing API Key!`; khác lỗi từ chối client ở host chính |

Các biến môi trường proxy thông dụng không được đặt trong môi trường chạy
kiểm tra. Các phản hồi JSON HTTP 401 cho thấy máy kết nối được tới dịch vụ;
không phải lỗi DNS, timeout hoặc lỗi tải giao diện HeraMind.

### Phân tích các phương án

1. **Sai endpoint:** endpoint chính khớp
   [hướng dẫn OpenClaw của AgentRouter](https://agentrouter.org/docs/openclaw.html).
   Sai dấu `/v1` không giải thích được cùng mã từ chối client ở cả ba đường dẫn.
2. **Sai giao thức:** cấu hình OpenAI phù hợp với model GPT. Thử đường dẫn
   Anthropic với khóa giả cũng gặp lớp từ chối client; điều này không xác nhận
   khóa thật hoặc model thật hoạt động trên Anthropic.
3. **Sai khóa/quyền tài khoản:** chưa loại trừ được. Không có khóa, khóa giả
   và khóa đang lưu đều gặp cùng loại lỗi, nên phản hồi này không thể phân biệt
   khóa đúng/sai. Cần kiểm chứng khóa bằng một client được AgentRouter hỗ trợ.
4. **Sai Model ID hoặc hết quota:** vẫn cần kiểm tra sau khi vượt qua bước
   xác thực client. `GET /v1/models` không gửi Model ID vẫn bị chặn, nên đổi
   tên model chưa xử lý được nguyên nhân hiện tại.
5. **Header nhận diện client:** khả năng phù hợp nhất với mã lỗi. User-Agent
   trung thực của HeraMind không giải quyết được phép thử không có khóa hợp lệ.
   [Báo cáo thử nghiệm OpenCodex #2717](https://github.com/lidge-jun/opencodex/issues/2717)
   ghi nhận client fingerprint ảnh hưởng kết quả. Đây là báo cáo của dự án
   khác, không phải cam kết hỗ trợ HeraMind từ AgentRouter.
6. **Endpoint `co.agentrouter.org`:**
   [tài liệu tại portal này](https://co.agentrouter.org/portal/guide) công bố
   URL OpenAI `https://co.agentrouter.org/v1`. Chỉ thử không có thông tin xác
   thực; chưa kiểm chứng khóa/tài khoản hiện tại được cấp cho portal đó.
   Không thể kết luận hai host dùng chung khóa, quota hay quyền truy cập.
7. **Proxy trung gian:** tự dựng proxy không tự cấp quyền client từ upstream.
   Proxy bên thứ ba nhận cả khóa và nội dung yêu cầu; chưa kiểm chứng một
   proxy nào với tài khoản này.

Các báo cáo trực tiếp [AgentRouter #37](https://github.com/agentrouter-org/docs/issues/37)
và [#21](https://github.com/agentrouter-org/docs/issues/21) cũng ghi nhận khóa
hoạt động trong client được hỗ trợ nhưng bị từ chối trong client tùy chỉnh.
Kết quả hiện tại củng cố giả thuyết lớp nhận diện client chặn trước khi có thể
kiểm chứng khóa/model; không chứng minh khóa hiện tại hợp lệ.

### Thứ tự xử lý

1. Xác định portal đã cấp API key và endpoint được cấp cho tài khoản.
2. Thử cùng khóa và model bằng một client có trong tài liệu của portal đó.
3. Nếu client chính thức hoạt động còn HeraMind bị chặn, yêu cầu AgentRouter
   hỗ trợ HeraMind hoặc cung cấp endpoint/header chính thức cho client tùy chỉnh.
4. Sau khi client được chấp nhận, kiểm tra danh sách model, quota và khả năng
   phản hồi thực tế trước khi kích hoạt backend.

Nội dung có thể gửi hỗ trợ, không kèm khóa API:

> HeraMind 0.9.24 receives HTTP 401 `unauthorized_client_error` from
> `https://agentrouter.org/v1`. Model discovery and a minimal chat request
> are rejected. Please confirm whether custom HeraMind clients are supported
> and provide an approved endpoint and client identification requirements.
