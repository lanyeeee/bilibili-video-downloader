pub trait AnyhowErrorToStringChain {
    /// 将 `anyhow::Error` 转换为chain格式  
    /// # Example  
    /// 0: error message\
    /// 1: error message\
    /// 2: error message
    fn to_string_chain(&self) -> String;
}

impl AnyhowErrorToStringChain for anyhow::Error {
    fn to_string_chain(&self) -> String {
        use std::fmt::Write;
        self.chain()
            .enumerate()
            .fold(String::new(), |mut output, (i, e)| {
                let _ = writeln!(output, "{i}: {e}");
                output
            })
    }
}
