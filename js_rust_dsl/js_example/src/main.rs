use js_macro::js;

fn main() {
    println!("--- Running JS-like script ---");
    js! {
        console.log("--- Functions ---");
        function greet(x) {
            console.log(x);
        }
        function world() {
            return "Hello";
        }
        let h = world();
        let words = h + " World";
        greet(words);

        function add3(a,b,c) {
            return a + b + c;
        }
        let number = add3(12, 11, 10);
        console.log("Number: " + number);
        console.log("--- Control Flow: ---");
        if (5 > 3) {
            console.log("If statement works!");
        }

        console.log("--- For loop ---");
        for (let i = 0; i < 5; i = i + 1) {
            console.log(i + " is less than 5");
        }

        console.log("--- While loop ---");
        let j = 0;
        while (j < 3) {
            console.log(j);
            j = j + 1;
        }

        console.log("--- Do-while loop ---");
        let k = 0;
        do {
            console.log("k = " + k);
            k = k + 1;
        } while (k < 2);

        console.log("--- Array example ---");
        let my_array = [1, "two", true];
        console.log("Initial array:", my_array);
        //console.log("value at index 2: " + my_array[2])
        my_array.push(4);
        console.log("After push(4):", my_array);
        let popped_value = my_array.pop();
        console.log("Popped value:", popped_value);
        console.log("After pop():", my_array);
        let numbers = [1, 2, 3, 4];
        let doubled = numbers.map(function(x) {
            return x * 2;
        });
        console.log("Original array:", numbers);
        console.log("Doubled after map():", doubled);

        console.log("--- Object example ---");
        _const person = {
            firstName: "John",
            lastName: "Doe",
            age: 0,
            gesture: function(x, y, z) {
                console.log(x + " " + y + " " + z);
            },
            setAge: function(x) {
                this.age = x;
            },
            info: function info(x) {
                console.log( this.firstName + " " + this.lastName + " " + this.age);
            },
            constructor: function(
                fistName,
                lastName,
                age
            ) {
                this.age = 22;
                console.log(this.age);
                let new_person = {
                    firstName: firstName,
                    lastName: lastName,
                    age: age,
                    gesture: this.gesture,
                    info: this.info,
                };
                console.log(firstName, lastName, age);
                return new_person;
            }
        };
        person.gesture("Hello!", "Fly", "Fox");
        person.setAge(21);
        person.info();

        let new_person = person.constructor("Blue", "Monkey", 22);
        console.log(new_person.age);
        new_person.info();
    }
    println!("--- Script finished ---");

    // // You can also directly interact with JsValue
    // use js_runtime::JsValue;
    //
    // let js_num: JsValue = 5.0.into();
    // let js_str: JsValue = "Rust".into();
    // let result = js_num.add(&js_str);
    // println!("Direct JsValue addition (5 + \"Rust\"): {}", result);
    //
    // let js_bool_true: JsValue = true.into();
    // let js_bool_false: JsValue = false.into();
    // let js_null: JsValue = JsValue::Null;
    // let js_undefined: JsValue = JsValue::Undefined;
    // let js_zero: JsValue = 0.0.into();
    // let js_empty_str: JsValue = "".to_string().into();
    //
    // println!("Truthiness of true: {}", js_bool_true.to_bool());
    // println!("Truthiness of false: {}", js_bool_false.to_bool());
    // println!("Truthiness of null: {}", js_null.to_bool());
    // println!("Truthiness of undefined: {}", js_undefined.to_bool());
    // println!("Truthiness of 0: {}", js_zero.to_bool());
    // println!("Truthiness of empty string: {}", js_empty_str.to_bool());
    // println!("Truthiness of 10: {}", JsValue::Number(10.0).to_bool());
    // println!("Truthiness of 'hello': {}", JsValue::String("hello".to_string()).to_bool());
}
