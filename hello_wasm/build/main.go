package main

import (
	"fmt"
)

func main() {
	// 	f, err := os.Create("hello.go")
	// 	if err != nil {
	// 		panic(err)
	// 	}
	// 	defer f.Close()
	//
	// 	if _, err := f.WriteString("hello"); err != nil {
	// 		panic(err)
	// 	}

	// f, err := os.ReadFile("1.txt")
	// if err != nil {
	// 	panic(err)
	// }
	// fmt.Println(string(f))

	fmt.Println("hello from golang")
}

// This function is exported to JavaScript, so can be called using
// exports.multiply() in JavaScript.
// export multiply
func multiply(x, y int) int {
	return x * y
}
