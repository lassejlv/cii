#[cfg(test)]
mod tests {
    use std::process::Command;

    #[test]
    fn interpret_block() {
        let output =
            Command::new("cargo")
            .arg("run")
            .arg("./src/tests/cases/block.jlox")
            .output()
            .unwrap();
        let lines = std::str::from_utf8(output.stdout.as_slice())
            .unwrap()
            .split("\n")
            .collect::<Vec<&str>>();

        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], "3");
        assert_eq!(lines[1], "3");
    }

    #[test]
    fn interpret_while() {
        let output =
            Command::new("cargo")
            .arg("run")
            .arg("./src/tests/cases/while.jlox")
            .output()
            .unwrap();
        let lines = std::str::from_utf8(output.stdout.as_slice())
            .unwrap()
            .split("\n")
            .collect::<Vec<&str>>();

        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], "1");
        assert_eq!(lines[1], "0");
    }

    #[test]
    fn interpret_while_math() {
        let output =
            Command::new("cargo")
            .arg("run")
            .arg("./src/tests/cases/while_math.jlox")
            .output()
            .unwrap();
        let lines = std::str::from_utf8(output.stdout.as_slice())
            .unwrap()
            .split("\n")
            .collect::<Vec<&str>>();

        assert_eq!(lines.len(), 11);
        assert_eq!(lines[0], "10");
        assert_eq!(lines[1], "90");
        assert_eq!(lines[2], "720");
        assert_eq!(lines[3], "5040");
        assert_eq!(lines[4], "30240");
        assert_eq!(lines[5], "151200");
        assert_eq!(lines[6], "604800");
        assert_eq!(lines[7], "1814400");
        assert_eq!(lines[8], "3628800");
        assert_eq!(lines[9], "3628800");
    }

    #[test]
    fn interpret_for_loop() {
        let output =
            Command::new("cargo")
            .arg("run")
            .arg("./src/tests/cases/forloop.jlox")
            .output()
            .unwrap();
        let lines = std::str::from_utf8(output.stdout.as_slice())
            .unwrap()
            .split("\n")
            .collect::<Vec<&str>>();

        let mut fibo = vec![];
        let mut a = 0;
        let mut b = 1;
        let mut temp;
        for _i in 0..21 {
            fibo.push(a);
            temp = b;
            b = a + b;
            a = temp;
        }

        assert_eq!(lines.len(), fibo.len() + 1);
        for i in 0..fibo.len() {
            assert_eq!(lines[i], fibo[i].to_string());
        }
    }
}
