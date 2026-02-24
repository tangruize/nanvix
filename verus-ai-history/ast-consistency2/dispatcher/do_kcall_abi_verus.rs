pub fn do_kcall_abi(number: u32, arg0: u32, arg1: u32, arg2: u32, arg3: u32) -> (encoded: i64)
    ensures ({
        let args: DispatchArgs = DispatchArgs { number, arg0, arg1, arg2, arg3 };
        exists|r: DispatchResultView| #![auto]
            spec_result_wf(r)
            && encoded as int == spec_encode_result(r)
            && (spec_classify_kcall(number) =~= DispatchCategory::LocalTerminal
                    ==> !r.is_success)
            && spec_dispatch_result_constrained(spec_classify_kcall(number), r)
    }),
{
    let args: DispatchArgs = DispatchArgs::new(number, arg0, arg1, arg2, arg3);
    let result: DispatchResult = do_kcall(args);
    proof {
        lemma_encode_result_is_value(result@);
    }
    encode_result(&result)
}
