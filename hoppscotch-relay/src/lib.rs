pub mod error;
pub mod interop;
pub mod relay;
pub mod util;

pub use error::{RelayError, RelayResult};
pub use interop::{RequestWithMetadata, ResponseWithMetadata};

pub fn run(req: RequestWithMetadata) -> RelayResult<ResponseWithMetadata> {
    relay::run(req)
}

pub fn cancel(req_id: usize) {
    relay::cancel(req_id)
}
