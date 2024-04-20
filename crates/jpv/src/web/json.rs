use axum::http::header::{self, HeaderValue};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use bytes::{BufMut, BytesMut};
use musli::mode::Text;
use musli::Encode;
use musli_json::Encoding;

const ENCODING: Encoding = Encoding::new();

pub(super) struct Json<T>(pub(super) T);

impl<T> IntoResponse for Json<T>
where
    T: Encode<Text>,
{
    fn into_response(self) -> Response {
        // Use a small initial capacity of 128 bytes like serde_json::to_vec
        // https://docs.rs/serde_json/1.0.82/src/serde_json/ser.rs.html#2189
        let mut buf = BytesMut::with_capacity(128).writer();

        match ENCODING.to_writer(&mut buf, &self.0) {
            Ok(()) => (
                [(
                    header::CONTENT_TYPE,
                    HeaderValue::from_static(mime::APPLICATION_JSON.as_ref()),
                )],
                buf.into_inner().freeze(),
            )
                .into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                [(
                    header::CONTENT_TYPE,
                    HeaderValue::from_static(mime::TEXT_PLAIN_UTF_8.as_ref()),
                )],
                err.to_string(),
            )
                .into_response(),
        }
    }
}
