pub struct ResponseError {
    request_format: u16,
    request_code: u16,
}

pub struct ResponseErrorExt {
    request_format: u16,
    request_code: u16,
    error_code: u16,
}
