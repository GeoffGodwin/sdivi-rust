// simple-go fixture: handlers.go
// Imports: 1 | Exports: 1
package main

import "fmt"

// Handle logs a handled event.
func Handle(event string) {
	fmt.Println("handled:", event)
}
