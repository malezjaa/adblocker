#[macro_export]
macro_rules! fwpm_transaction {
    ($engine:expr, $blck:block) => {{
        let status = FwpmTransactionBegin0($engine, 0);
        if status != 0 {
            anyhow::bail!("FwpmTransactionBegin0 failed: {:#010x}", status);
        }

        let result: anyhow::Result<()> = (|| $blck)();
        if result.is_err() {
            FwpmTransactionAbort0($engine);
            return result;
        }

        let status = FwpmTransactionCommit0($engine);
        if status != 0 {
            FwpmTransactionAbort0($engine);
            anyhow::bail!("FwpmTransactionCommit0 failed: {:#010x}", status);
        }
    }};
}