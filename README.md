# colonnade
tabular text

```plain
         Colonnade lets     As you can see, it supports text     If you want to 
        you format text      alignment, viewport width, and      colorize your  
            in columns.              column widths.              table, you'll  
                                                                 need to use the
                                                                 macerate       
                                                                 method.        

                           Two or more rows of columns makes                    
                                        a table.                                
```

## USAGE

```rust
extern crate colonnade;
use colonnade::{Alignment, Colonnade};

#[allow(unused_must_use)]
fn main() {
    // text to put in tabular form
    let text = vec![
        vec![
            "Colonnade lets you format text in columns.",
            "As you can see, it supports text alignment, viewport width, and column widths.",
            "If you want to colorize your table, you'll need to use the macerate method.",
        ],
        vec!["", "Two or more rows of columns makes a table.", ""],
    ];
    let mut colonnade = Colonnade::new(3, 80).unwrap(); // 3 columns of text in an 80-character viewport

    // configure the table a bit
    colonnade.left_margin(4);
    colonnade.columns[0].left_margin(8); // the first column should have a left margin 8 spaces wide
    colonnade.fixed_width(15);
    colonnade.columns[0].clear_limits(); // the central column has no fixed size limits
    colonnade.columns[0].alignment(Alignment::Right);
    colonnade.columns[1].alignment(Alignment::Center);
    colonnade.columns[2].alignment(Alignment::Left);
    colonnade.spaces_between_rows(1); // add a blank link between rows

    // now print out the table
    for line in colonnade.tabulate(&text).unwrap() {
        println!("{}", line);
    }
}
```
This produces
```plain
         Colonnade lets     As you can see, it supports text     If you want to 
        you format text      alignment, viewport width, and      colorize your  
            in columns.              column widths.              table, you'll  
                                                                 need to use the
                                                                 macerate       
                                                                 method.        

                           Two or more rows of columns makes                    
                                        a table.                                
```