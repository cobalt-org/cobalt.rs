mod source;

pub use source::*;

type Status = status::Status;
type Result<T, E = Status> = std::result::Result<T, E>;
