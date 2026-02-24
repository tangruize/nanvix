pub fn do_kcall_encoded(args: DispatchArgs) -> (pair: (DispatchResult, i64))
    ensures ({
        let result: DispatchResult = pair.0;
        let encoded: i64 = pair.1;
        // The result is well-formed.
        &&& result.wf()
        // The encoded i64 equals spec_encode_result of this specific result.
        &&& encoded as int == spec_encode_result(result@)
        // The result satisfies all do_kcall postconditions.
        &&& (spec_classify_kcall(args.number) =~= DispatchCategory::LocalTerminal
                ==> !result@.is_success)
        &&& spec_dispatch_result_constrained(spec_classify_kcall(args.number), result@)
        &&& ((args@.number == 1 || args@.number == 2)
                && result@.is_success ==> result@.value >= 0)
        &&& ((args@.number == 9 || args@.number == 24
                || args@.number == 27 || args@.number == 29
                || args@.number == 25 || args@.number == 20)
                && result@.is_success ==> result@.value == 0)
        &&& (args@.number == 26 && result@.is_success ==> result@.value >= 0)
        &&& (args@.number == 23 && result@.is_success ==> result@.value >= 0)
    }),
{
    let result: DispatchResult = do_kcall(args);
    proof {
        lemma_encode_result_is_value(result@);
    }
    let encoded: i64 = encode_result(&result);
    (result, encoded)
}
