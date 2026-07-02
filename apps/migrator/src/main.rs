use platform_domain::database::milestone1_tables;

fn main() {
    let tables = milestone1_tables();
    let tenant_scoped = tables.iter().filter(|table| table.tenant_scoped).count();

    println!("Migrator bootstrap");
    println!("Logical tables: {}", tables.len());
    println!("Tenant-scoped tables: {}", tenant_scoped);
    println!("First table: {}", tables[0].name);
}
