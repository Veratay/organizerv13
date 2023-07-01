struct Person {
    name:String,
    age:i32
}

impl Descriptable for Person {
    fn print_description(&self) {
        println!("Hey! im a person, my name is" + self.name + "and I am " + self.age + "years old")
    }
}

struct Animal {
    species:String,
    number_of_feet:i32,
    is_mammal:bool
}

type AnimalTuple = (String,i32,bool);

impl Descriptable for Animal {
    fn print_description(&self) {
        println!("Hey! I am an animal of species" + self.species + ", I have " + self.number_of_feet + "feet, and the statement I am a mammal is" + self.is_mammal);
    }
}

trait Descriptable {
    fn print_description(&self);
}

fn get_description(x: Descriptable) {
    x.print_description();
}

fn get_description_of_list(x: &Vec<Descriptable>) {
    for animal in x.iter() {
        animal.print_description();
    }
    //free(x)
}

fn main() {
    //compiler asks for 4 bytes
    let x:i32 = 42;
    //compiler asks for 12 bytes
    let random_person:Person = Person { name:String::from("bob"), age:100 };
    //compiler asks for 12 bytes
    let person1:Person = Person {name:String::from("alice"), age: 10 };

    let person_vip:SpecialPerson = SpecialPerson { name:String::from("oauinsdf"), age:32 };

    //i32, pointer, pointer,              i32
    // x,  string part of random_person   age part of random person

    //free(x, random_person, person1)

    let animal_one:Animal = Animal { species:"dog", number_of_feet:4, is_mammal:true };
    animal_one.print_description();
    Animal::print_description(&animal_one);

    let animal_two:Animal = Animal { species:"cat", number_of_feet:4, is_mammal:true };
    animal_two.print_description();

    let list:Vec<Descriptable> = vec![animal_one, animal_two, person_vip];
    get_description_of_list(&list);
    get_description_of_list(&list);

    //free (everything)
}

type SpecialPerson = Person;