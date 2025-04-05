pub use sea_orm_migration::prelude::*;

mod m20250213_100210_create_table_users;
mod m20250213_105425_create_table_posts;
mod m20250304_075607_create_table_messages;
mod m20250316_020000_create_table_reaction_types;
mod m20250316_020100_create_table_reactions;
mod m20250317_200000_create_table_groups;
mod m20250317_200100_create_table_group_members;
mod m20250317_200200_create_table_group_messages;
mod m20250317_200300_create_table_group_message_reads;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20250213_100210_create_table_users::Migration),
            Box::new(m20250213_105425_create_table_posts::Migration),
            Box::new(m20250304_075607_create_table_messages::Migration),
            Box::new(m20250316_020000_create_table_reaction_types::Migration),
            Box::new(m20250316_020100_create_table_reactions::Migration),
            Box::new(m20250317_200000_create_table_groups::Migration),
            Box::new(m20250317_200100_create_table_group_members::Migration),
            Box::new(m20250317_200200_create_table_group_messages::Migration),
            Box::new(m20250317_200300_create_table_group_message_reads::Migration),
        ]
    }
}
