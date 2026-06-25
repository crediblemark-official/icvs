# Syntax Reference — InstructCanvas (.icvs)

## Format Dasar

File `.icvs` adalah teks polos UTF-8 dengan struktur blok dan indentation. Setiap baris masuk ke dalam salah satu kategori:

1. **Komentar** — diawali `#`
2. **Block header** — diawali `[` dan diakhiri `]`
3. **Atribut** — baris terindentasi milik block header sebelumnya
4. **Baris kosong** — diabaikan

## Komentar

```plaintext
# Ini komentar
#project: "my-api"       # BUKAN komentar — ini metadata proyek
```

Baris yang diawali `#` adalah komentar. **Kecuali** `#project:` yang merupakan metadata khusus.

## Metadata Proyek

```plaintext
#project: "nama-proyek"
```

Seperti komentar tapi bukan — ini mendefinisikan nama proyek. Hanya satu per file.

## Include

```plaintext
[include: "path/to/file.icvs"]
```

Mengimpor file `.icvs` lain secara modular. Path relatif terhadap file sumber. Resolver mendeteksi circular include.

## Node

```plaintext
[node: <id>]
  type = <node_type>
  content = "<instruksi>"
  severity = <severity>       # hanya untuk type=rule
  trigger_on = <trigger>      # hanya untuk type=blocklist/allowlist
  if = $VARIABLE == "value"   # hanya untuk type=condition
    then = -> <node_id>
    else = -> <node_id>
```

### Node Types

| Type | Deskripsi | Atribut Kunci |
|---|---|---|
| `rule` | Aturan coding yang harus dipatuhi | `severity`, `content` |
| `blocklist` | Library/kode yang dilarang | `trigger_on`, `content` |
| `allowlist` | Library/kode yang diizinkan | `trigger_on`, `content` |
| `condition` | Logika kondisional berbasis env | `if`, `then`, `else` |
| `action` | Tindakan yang harus dijalankan | `content` |

### Severity

| Value | Makna |
|---|---|
| `must` | Wajib — agen HARUS mematuhi |
| `should` | Sebaiknya — rekomendasi kuat |
| `may` | Boleh — optional |

### Trigger On

| Value | Makna |
|---|---|
| `import` | Cek saat mengimpor library |
| `install` | Cek saat menginstal package |
| `run` | Cek saat menjalankan perintah |

### Condition

```plaintext
[node: deploy_check]
  type = condition
  if = $BRANCH == "main"
    then = -> run_deploy
    else = -> skip_deploy
```

Variabel environment menggunakan prefix `$`. Operator yang didukung: `==`, `!=`, `>=`, `<=`, `>`, `<`.

Node condition secara implisit membuat edge ke node `then` dan `else`.

## Edge

```plaintext
[edge: <source_id> -> <target_id>]
```

Mendefinisikan dependensi antar node. Source akan dieksekusi sebelum target. Graph harus berupa **Directed Acyclic Graph (DAG)** — cycle akan ditolak oleh validator.

## Target

```plaintext
[target: <tool_name>]
  resolve = [node_id, node_id, ...]
  ignore = [node_id, ...]
```

Menentukan node mana yang berlaku untuk tool tertentu.

| Atribut | Deskripsi |
|---|---|
| `resolve` | Daftar node yang termasuk untuk target ini |
| `ignore` | Daftar node yang dikecualikan (subset dari resolve) |

Contoh: Satu file `.icvs` bisa punya target berbeda untuk tool berbeda:

```plaintext
[target: claude]
  resolve = [coding_style, forbidden_libs, prod_rule]

[target: copilot]
  resolve = [coding_style]
  ignore = [forbidden_libs]
```

## Aturan Validasi

### Structural
- Setiap node harus punya `id` unique
- Edge harus mereferensi node yang ada
- Graph harus DAG (tidak boleh ada cycle)
- Atribut dalam block node/target harus terindentasi

### Naming
- Node ID: alfanumerik, underscore, hyphen (`^[a-zA-Z0-9_-]+$`)
- Target name: sama dengan node ID

### Merging (Include)
- Node dengan ID yang sama dari file berbeda → error duplicate
- Target dengan nama yang sama dari file berbeda → error duplicate
- Circular include → error

## Grammar EBNF

```ebnf
document     = { comment | include | node | edge | target | project } ;
comment      = "#" , { character } , newline ;
include      = "[include:" , string , "]" ;
node         = "[node:" , identifier , "]" , newline , { attribute } ;
edge         = "[edge:" , identifier , "->" , identifier , "]" ;
target       = "[target:" , identifier , "]" , newline , { target_attr } ;
project      = "#project:" , string ;

attribute    = ( "type" | "content" | "severity" | "trigger_on"
              | "if" | "then" | "else" ) , "=" , value ;
target_attr  = ( "resolve" | "ignore" ) , "=" , "[" , identifier , { "," , identifier } , "]" ;

value        = string | identifier | condition | arrow_ref ;
condition    = "$" , identifier , operator , string ;
arrow_ref    = "->" , identifier ;
operator     = "==" | "!=" | ">=" | "<=" | ">" | "<" ;
identifier   = ( letter | "_" ) , { letter | digit | "_" | "-" } ;
string       = "\"" , { character } , "\"" ;
```
