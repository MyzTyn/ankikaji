# AnkiKaji CLI

AnkiKaji is a simple command-line tool for managing Japanese study cards with SQLite. It supports importing YAML data, exporting to CSV, and automatic preprocessing (furigana via autoruby).

✨ Features

📥 Import YAML into SQLite (cards.yaml)

📤 Export CSV of unexported cards

🔁 Upsert logic (insert or update on conflict)

🏷 Flexible schema defined in jp-template.yaml

📝 Automatic annotation (furigana with autoruby)

# 🚀 Usage

1. Build
cargo build --release

2. Set environment variables \
```bash
export ANKIKAJI_DB=ankikaji.db
export ANKIKAJI_TABLE=cards
```

3. Import YAML
./target/release/ankikaji import-yaml \
This loads cards.yaml into your database.

4. Export CSV
./target/release/ankikaji export-csv \
This exports all unexported cards into export.csv and marks them as exported.

# 📂 Project Structure
```
src/
 ├── main.rs        # CLI entrypoint (clap commands)
 ├── db.rs          # Database helpers (import/export, transactions)
 ├── models.rs      # Card metadata, schema, YAML parsing
```

# ⚙️ Example Workflow

- Define your card schema in jp-template.yaml
- Add new cards into cards.yaml
- Run ankikaji import-yaml
- Export to CSV for Anki with ankikaji export-csv
