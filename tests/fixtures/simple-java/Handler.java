// simple-java fixture: Handler.java
// Imports: 1 | Exports: 1
import java.util.List;

public class Handler {
    public void handle(List<String> items) {
        items.forEach(System.out::println);
    }
}
