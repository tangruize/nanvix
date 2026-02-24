    pub fn new(number: u32, arg0: u32, arg1: u32, arg2: u32, arg3: u32) -> (result: DispatchArgs)
        ensures
            result@.number == number as nat,
            result@.arg0 == arg0 as nat,
            result@.arg1 == arg1 as nat,
            result@.arg2 == arg2 as nat,
            result@.arg3 == arg3 as nat,
            result.wf(),
    {
        DispatchArgs { number, arg0, arg1, arg2, arg3 }
    }
