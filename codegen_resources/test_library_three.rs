mod TestLibrary {
    pub trait Found {
    }

    #[singleton("hello_string")]
    fn this_one() -> Option<&'static str> {
        Some("hello")
    }

    impl Found for One {
    }

    impl Found for Four {
    }


    impl One {
    }

    #[derive(Default, Debug)]
    #[singleton(four)]
    pub struct Four {
        two: String
    }

    #[derive(Default, Debug)]
    pub struct One {
        pub two: String,
        #[autowired]
        pub four: Four
    }

    #[singleton(Once)]
    #[derive(Default, Debug)]
    pub struct Once {
        // pub(crate) fns: Vec<Box<dyn FnOnce(())>>
    }
}

