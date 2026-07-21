use crate::customer_contact_book::error::PhoneNumberError;

#[derive(Debug)]
pub struct PhoneNumber(String);

impl PhoneNumber {
    pub fn parse(input_number: &str) -> Result<Self, PhoneNumberError> {
        let trimmed = input_number.trim();

        if trimmed.is_empty() {
            return Err(PhoneNumberError::Empty);
        }

        // Один проход по строке собирает только цифры. Это проще и устойчивее,
        // чем перечислять все допустимые разделители вроде пробелов, скобок и
        // дефисов: новый визуальный разделитель не сломает валидный номер.
        let digits: String = trimmed.chars().filter(|ch| ch.is_ascii_digit()).collect();

        // На этом шаге строка уже состоит только из цифр, поэтому проверка
        // сводится к длине и префиксу. Результат всегда приводится к цифрам без
        // плюса, а плюс добавляется один раз в самом конце.
        let normalized_digits = if digits.len() == 12 && digits.starts_with("375") {
            digits
        } else if digits.len() == 11 && digits.starts_with("80") {
            // Локальный белорусский формат `80XXXXXXXXX` переводим в
            // международный `375XXXXXXXXX`, отбрасывая первые две цифры `80`.
            format!("375{}", &digits[2..])
        } else {
            return Err(PhoneNumberError::InvalidFormat);
        };

        // Единственная точка создания значения. После нее `PhoneNumber` всегда
        // хранится одинаково, что упрощает сравнение, хеширование и вывод.
        Ok(Self(format!("+{}", normalized_digits)))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for PhoneNumber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
