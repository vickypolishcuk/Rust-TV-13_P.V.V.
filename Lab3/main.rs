use std::fs::File;
use std::io::{BufRead, stdin};
use serde::{Serialize, Deserialize};
use std::io::{self, BufReader, Write};

// Макроси для автоматичної генерації необхідних імплементацій для структур
// Структури для роботи з даними та використання їх у файлі
#[derive(Debug, Serialize, Deserialize)]
struct Task {
    id: u32,
    description: String,
    completed: bool,
}

// Debug для форматування і виведення структури в текстовому вигляді
// Serialize, Deserialize для збереження даних у файл
#[derive(Debug, Serialize, Deserialize)]
struct User {
    username: String,
    password: String,
    tasks: Vec<Task>,
}

// Реалізація користувача
impl User {
    // Функція створення нового користувача
    fn new(username: String, password: String) -> Self {
        User {
            username,
            password,
            tasks: vec![],
        }
    }

    // Функція перевірки правильності введеного паролю
    fn authenticate(&self, password: &str) -> bool {
        password == self.password
    }

    // Функція створення нового завдання
    fn add_task(&mut self, description: String) {
        // Визначення ідентифікатора нового завдання
        let id = self.tasks.len() as u32 + 1;
        // Додавання нового завдання у вектор завдань користувача
        self.tasks.push(Task {
            id,
            description,
            completed: false, // за замовчуванням нове завдання має статус "не виконано"
        });
    }

    // Функція редагування завдання
    fn edit_task(&mut self, id: u32, new_description: String) {
        if let Some(task) = self.tasks.iter_mut().find(|task| task.id == id) {
            task.description = new_description;
        }
    }

    // Функція видалення завдання
    fn remove_task(&mut self, id: u32) {
        // retain повертає вектор з тими завданнями, id яких не збігаються з переданим
        self.tasks.retain(|task| task.id != id);
        // Цикл для оновлення ідентифікаторів завдань
        for (index, task) in self.tasks.iter_mut().enumerate() {
            task.id = (index as u32) + 1;
        }
    }

    // Функція позначення обраного завдання як виконане
    fn mark_task_completed(&mut self, id: u32) {
        if let Some(task) = self.tasks.iter_mut().find(|task| task.id == id) {
            task.completed = true;
        }
    }

    // Функція виведення всіх завдань
    fn list_tasks(&self) {
        println!("Список завдань:");
        for task in &self.tasks {
            println!("ID: {}, Опис: {}, Статус: {}", task.id, task.description, if task.completed { "Виконано" } else { "Не виконано" });
        }
    }

    // Функція збереження даних до файлу
    fn save_to_file(&self, filename: &str) -> io::Result<()> {
        let serialized_data = serde_json::to_string(&self)?; // серіалізація даних
        let mut file = File::create(filename)?; // створення нового файлу
        file.write_all(serialized_data.as_bytes())?; // запис оброблених даних у файл
        Ok(())
    }

    // Функція завантаження даних з файлу
    fn load_from_file(filename: &str) -> io::Result<User> {
        let file = File::open(filename)?; // відкриття файлу
        let reader = BufReader::new(file); // змінна для читання з файлу
        let user: User = serde_json::from_reader(reader)?; // десеріалізація даних в змінну user
        Ok(user) // повернення даних
    }

}


// Функція для обробки введених значень користувачем
fn input(prompt: &str) -> String {
    println!("{}", prompt);
    let mut input = String::new();
    stdin().lock().read_line(&mut input).expect("Не вдалося прочитати рядок");
    input.trim().to_string()
}


fn main() {
    
    let filename = "tasks.txt";

    // Завантаження або створення нового користувача
    let mut user = match User::load_from_file(filename) {
        Ok(u) => u, // якщо користувач є, повернути його дані
        Err(_) => { // якщо користувача немає, створити нового
            println!("Користувача не знайдено. Створимо нового.");
            // Перевірка введення імені користувача
            let username = loop {
                let input_username = input("Введіть ім'я користувача:");
                if input_username.trim().is_empty() {
                    println!("Ім'я користувача не може бути порожнім. Спробуйте ще раз.");
                } else {
                    break input_username;
                }
            };

            // Перевірка введення пароля
            let password = loop {
                let input_password = input("Введіть пароль:");
                if input_password.trim().is_empty() {
                    println!("Пароль не може бути порожнім. Спробуйте ще раз.");
                } else if input_password == "exit" {
                    println!("Пароль не може бути 'exit'. Спробуйте ще раз.");
                } else {
                    break input_password;
                }
            };
            User::new(username, password)
        }
    };

    // Авторизація користувача
    loop {
        let password = input("Введіть пароль для авторизації (або введіть 'exit' для виходу):");
        if password == "exit" { // можна вийти з програми за допомогою exit
            println!("Вихід з програми.");
            return;
        }

        if user.authenticate(&password) {
            break; // пароль вірний, вихід з циклу
        } else {
            println!("Невірний пароль. Спробуйте ще раз.");
        }
    }

    loop {
        // Меню користувача
        println!("\nМеню:");
        println!("1. Додати завдання");
        println!("2. Редагувати завдання");
        println!("3. Видалити завдання");
        println!("4. Позначити завдання як виконане");
        println!("5. Показати список завдань");
        println!("6. Зберегти та вийти");

        let choice = input("Виберіть дію:");

        match choice.as_str() {
            "1" => {
                let description = input("Опис завдання:");
                user.add_task(description);
            }
            "2" => {
                if user.tasks.len() == 0 {
                    println!("У вас немає завдань для редагування");
                } else {
                    let id = input("ID завдання для редагування:");
                    // Спроба конвертації введеного рядка в u32
                    match id.parse::<u32>() {
                        Ok(id) => {
                            // Перевірка правильності введеного id
                            if let Some(_) = user.tasks.iter().find(|task| task.id == id) {
                                let new_description = input("Новий опис завдання:");
                                user.edit_task(id, new_description);
                            } else {
                                println!("Завдання з таким ID не знайдено.");
                            }
                        }
                        Err(_) => {
                            println!("ID було введено неправильно");
                        }
                    }
                }
            }
            "3" => {
                if user.tasks.len() == 0 {
                    println!("У вас немає завдань для видалення");
                } else {
                    let id = input("ID завдання для видалення:");
                    // Спроба конвертації введеного рядка в тип u32
                    match id.parse::<u32>() {
                        Ok(id) => {
                            // Перевірка правильності введеного id
                            if let Some(_) = user.tasks.iter().find(|task| task.id == id) {
                                user.remove_task(id);
                            } else {
                                println!("Завдання з таким ID не знайдено.");
                            }
                        }
                        Err(_) => {
                            println!("ID було введено неправильно");
                        }
                    }                    
                }
            }
            "4" => {
                if user.tasks.len() == 0 {
                    println!("У вас немає завдань для позначення");
                } else {
                    let id = input("ID завдання для позначення як виконане:");
                    // Спроба конвертації введеного рядка в тип u32
                    match id.parse::<u32>() {
                        Ok(id) => {
                            // Перевірка правильності введеного id
                            if let Some(_) = user.tasks.iter().find(|task| task.id == id) {
                                user.mark_task_completed(id);
                            } else {
                                println!("Завдання з таким ID не знайдено.");
                            }
                        }
                        Err(_) => {
                            println!("ID було введено неправильно");
                        }
                    }
                }
            }
            "5" => {
                if user.tasks.len() == 0 {
                    println!("У вас ще немає завдань");
                } else {
                    user.list_tasks();
                }
            }
            "6" => {
                user.save_to_file(filename).unwrap();
                println!("Завдання збережено. Вихід.");
                break;
            }
            "exit" => {
                break;
            }
            _ => println!("Невірний вибір. Спробуйте ще раз."),
        }
    }
}
