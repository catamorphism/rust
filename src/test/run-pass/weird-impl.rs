import to_str::to_str;

    type hashmap__ = {
        mut count: uint
    };


    impl of to_str for hashmap__ {
       fn to_writer(wr: io::writer) {
            if self.count == 0u {
                wr.write_str("{}");
                ret;
            }

            wr.write_str("{ ");
            let mut first = true;
            /*
            for self.each_entry |entry| {
                if !first {
                    wr.write_str(", ");
                }
                first = false;
                wr.write_str(entry.key.to_str());
                wr.write_str(": ");
                wr.write_str((copy entry.value).to_str());
            };
            */
            wr.write_str(" }");
        }

        fn to_str() -> ~str {
          // do io::with_str_writer |wr| { self.to_writer(wr) }
          let wr = fail;
          self.to_writer(wr);
        }
    }

