use rwfw_core::migration::Migration;

pub fn migrations() -> Vec<Migration> {
    vec![
        Migration::new(
            "202605060001",
            "create_blog_schema",
            include_str!("202605060001_create_blog_schema.sql"),
        ),
        Migration::new(
            "202605060002",
            "create_posts_table",
            include_str!("202605060002_create_posts_table.sql"),
        ),
    ]
}
