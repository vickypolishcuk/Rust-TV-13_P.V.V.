use std::io;
use std::str::FromStr;

// Операції калькулятора
enum Operation {
    Add,
    Subtract,
    Multiply,
    Divide,
}

// Реалізація зчитування операції
impl FromStr for Operation {
    type Err = ();

    fn from_str(input: &str) -> Result<Operation, Self::Err> {
        match input.trim() {
            "+" => Ok(Operation::Add),
            "-" => Ok(Operation::Subtract),
            "*" => Ok(Operation::Multiply),
            "/" => Ok(Operation::Divide),
            _ => Err(()),
        }
    }
}

// Функція для виконання обраної операції
fn perform_operation(num1: f64, op: Operation, num2: f64) -> Result<f64, String> {
    match op {
        Operation::Add => Ok(num1 + num2),
        Operation::Subtract => Ok(num1 - num2),
        Operation::Multiply => Ok(num1 * num2),
        Operation::Divide => {
            if num2 == 0.0 {
                Err("Помилка: ділення на нуль!".to_string())
            } else {
                Ok(num1 / num2)
            }
        }
    }
}

// Обробка виразу з дотриманням пріоритетів операцій
fn evaluate_expression(expression: &str) -> Result<f64, String> {
    let mut numbers: Vec<f64> = Vec::new();
    let mut operations: Vec<Operation> = Vec::new();

    let tokens: Vec<&str> = expression.split_whitespace().collect();
    let mut i = 0;

    // Проходимо по всіх токенах у виразі
    while i < tokens.len() {
        let token = tokens[i];

        // Якщо це число, додаємо до стеку чисел
        if let Ok(num) = token.parse::<f64>() {
            numbers.push(num);
        } else if let Ok(op) = token.parse::<Operation>() {
            // Якщо це оператор, перевіряємо наступний елемент
            if i + 1 < tokens.len() {
                if let Ok(next_num) = tokens[i + 1].parse::<f64>() {
                    if matches!(op, Operation::Multiply | Operation::Divide) {
                        // Виконуємо операції множення або ділення відразу
                        let prev_num = numbers.pop().unwrap();
                        let result = perform_operation(prev_num, op, next_num)?;
                        numbers.push(result);
                    } else {
                        // Якщо це додавання чи віднімання, записуємо в стек операцію і йдемо далі
                        numbers.push(next_num);
                        operations.push(op);
                    }
                    i += 1; // Пропускаємо наступне число, бо вже його опрацювали
                } else {
                    return Err("Помилка: очікується число після оператора".to_string());
                }
            } else {
                return Err("Помилка: очікується число після оператора".to_string());
            }
        } else {
            return Err(format!("Невідомий токен: {}", token));
        }
        i += 1;
    }

    // Виконуємо операції додавання та віднімання після обробки множення та ділення
    while let Some(op) = operations.pop() {
        let num2 = numbers.pop().unwrap();
        let num1 = numbers.pop().unwrap();
        let result = perform_operation(num1, op, num2)?;
        numbers.push(result);
    }

    // Повертаємо фінальний результат
    if numbers.len() == 1 {
        Ok(numbers.pop().unwrap())
    } else {
        Err("Помилка обчислення".to_string())
    }
}

// Польська нотація
fn poland_notation(expression: &str) -> Result<f64, String> {
    let mut stack: Vec<f64> = Vec::new();

    for token in expression.split_whitespace() {
        match token {
            "+" => {
                let b = stack.pop().ok_or("Недостатньо операндів")?;
                let a = stack.pop().ok_or("Недостатньо операндів")?;
                stack.push(a + b);
            }
            "-" => {
                let b = stack.pop().ok_or("Недостатньо операндів")?;
                let a = stack.pop().ok_or("Недостатньо операндів")?;
                stack.push(a - b);
            }
            "*" => {
                let b = stack.pop().ok_or("Недостатньо операндів")?;
                let a = stack.pop().ok_or("Недостатньо операндів")?;
                stack.push(a * b);
            }
            "/" => {
                let b = stack.pop().ok_or("Недостатньо операндів")?;
                let a = stack.pop().ok_or("Недостатньо операндів")?;
                if b == 0.0 {
                    return Err(String::from("Помилка: ділення на нуль!"));
                }
                stack.push(a / b);
            }
            _ => {
                let num = token.parse::<f64>().map_err(|_| "Неправильний формат числа")?;
                stack.push(num);
            }
        }
    }

    if stack.len() == 1 {
        Ok(stack.pop().unwrap())
    } else {
        Err(String::from("Помилка: неправильний вираз."))
    }
}

fn main() {
    let mut memory: f64 = 0.0;

    loop {
        println!("Оберіть тип калькулятора:");
        println!("1: Класичний калькулятор");
        println!("2: Калькулятор у польській нотації");
        println!("Введіть memory для отримання пам'яті");
        println!("Введіть 'exit' для виходу.");

        let mut choice = String::new();
        io::stdin().read_line(&mut choice).expect("Не вдалося прочитати рядок");
        let choice = choice.trim();

        if choice.eq_ignore_ascii_case("exit") {
            break;
        }

        match choice {
            "1" => {
                println!("Введіть вираз у форматі: <число> <операція> <число>");
                let mut input = String::new();
                io::stdin().read_line(&mut input).expect("Не вдалося прочитати рядок");
                let input = input.trim();

                // Зміна вхідного виразу, якщо введено "memory"
                let mut expression = input.to_string();
                if input.starts_with("memory") {
                    // Додаємо значення з пам'яті у вираз
                    let rest_of_expression = &input[6..].trim(); // Отримуємо частину виразу після "memory"
                    expression = format!("{} {}", memory, rest_of_expression);
                }


                // Обробляємо вираз
                match evaluate_expression(&expression) {
                    Ok(result) => {
                        println!("Результат: {}", result);
                        memory = result; // Записуємо результат у пам'ять
                    },
                    Err(e) => println!("{}", e),
                }

            }
            "2" => {
                println!("Введіть вираз у польській нотації:");
                let mut input = String::new();
                io::stdin().read_line(&mut input).expect("Не вдалося прочитати рядок");
                let input = input.trim();

                // Зміна вхідного виразу, якщо введено "memory"
                let mut expression = input.to_string();
                if expression.starts_with("memory") {
                    // Додаємо значення з пам'яті в вираз
                    let rest_of_expression = &expression[6..].trim(); // Отримуємо частину виразу після "memory"
                    expression = format!("{} {}", memory, rest_of_expression);
                }

                match poland_notation(&expression) {
                    Ok(result) => {
                        println!("Результат: {}", result);
                        memory = result;
                    }
                    Err(err) => println!("{}", err),
                }
            }
            "memory" =>{
                println!("Поточне значення в пам'яті: {}", memory);
            }
            _ => println!("Невірний вибір. Спробуйте ще раз."),
        }
    }
}
