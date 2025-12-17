#!/usr/bin/env python3
"""
Script to apply additional derives and attributes to generated SeaORM entities.
Usage:
  Type-level: python3 apply_derives.py <file_path> <type_name> <derives> [attributes]
  Field-level: python3 apply_derives.py <file_path> <type_name> --field <field_name> <attribute>

Examples:
  python3 apply_derives.py crates/entities/src/sea_orm_active_enums.rs OauthProvider "ToSchema" "#[serde(rename_all = \"lowercase\")]"
  python3 apply_derives.py crates/entities/src/user.rs User --field password "#[serde(skip)]"
"""

import sys
import re
from pathlib import Path
from typing import Optional


def apply_field_attribute(
    file_path: str, type_name: str, field_name: str, attribute: str
):
    """Apply an attribute to a specific field in a Rust struct."""
    file = Path(file_path)

    if not file.exists():
        print(f"Error: File {file_path} not found", file=sys.stderr)
        sys.exit(1)

    content = file.read_text()
    lines = content.split("\n")

    # Find the struct definition
    struct_pattern = rf"pub\s+struct\s+{type_name}\b"
    struct_match = None
    struct_line_idx = None

    for i, line in enumerate(lines):
        if re.search(struct_pattern, line):
            struct_match = line
            struct_line_idx = i
            break

    if not struct_match:
        print(f"Error: Struct {type_name} not found in {file_path}", file=sys.stderr)
        sys.exit(1)

    # Find the field within the struct
    # Look for the opening brace and then search for the field
    brace_found = False
    field_line_idx = None
    if not struct_line_idx:
        return

    for i in range(struct_line_idx, len(lines)):
        line = lines[i]
        if "{" in line:
            brace_found = True
            continue
        if brace_found and "}" in line:
            break
        if brace_found:
            # Look for field definition: pub field_name: Type,
            field_pattern = rf"\s*pub\s+{field_name}\s*:"
            if re.search(field_pattern, line):
                field_line_idx = i
                break

    if field_line_idx is None:
        print(
            f"Error: Field {field_name} not found in struct {type_name}",
            file=sys.stderr,
        )
        sys.exit(1)

    # Check if the attribute already exists on the previous line(s)
    for i in range(max(0, field_line_idx - 3), field_line_idx):
        if attribute in lines[i]:
            print(f"Attribute {attribute} already exists for field {field_name}")
            return

    # Find the correct position to insert the attribute
    # Look backwards from the field to find any existing attributes
    insert_idx = field_line_idx
    for i in range(field_line_idx - 1, max(-1, field_line_idx - 5), -1):
        line = lines[i].strip()
        if line.startswith("#[") and line.endswith("]"):
            # Found an existing attribute, insert after it
            insert_idx = i + 1
            break
        elif line and not line.startswith("//"):
            # Found non-attribute, non-comment line, insert before field
            insert_idx = field_line_idx
            break

    # Insert the attribute with proper indentation
    field_line = lines[field_line_idx]
    field_indent = len(field_line) - len(field_line.lstrip())
    indented_attribute = " " * field_indent + attribute

    lines.insert(insert_idx, indented_attribute)

    # Write back
    file.write_text("\n".join(lines))
    print(
        f"Successfully applied attribute {attribute} to field {field_name} in {type_name}"
    )


def apply_derives(
    file_path: str, type_name: str, derives: str, attributes: Optional[str] = None
):
    """Apply derives and attributes to a type in a Rust file."""
    file = Path(file_path)

    if not file.exists():
        print(f"Error: File {file_path} not found", file=sys.stderr)
        sys.exit(1)

    content = file.read_text()

    # Pattern to match the type definition and its attributes
    # We need to find: #[derive(...)] #[sea_orm(...)] pub enum/struct TypeName
    type_pattern = rf"(pub\s+(enum|struct)\s+{type_name}\b)"

    match = re.search(type_pattern, content)
    if not match:
        print(f"Warning: Type {type_name} not found in {file_path}", file=sys.stderr)
        sys.exit(1)

    type_start = match.start()

    # Look backwards for #[derive(...)] and #[sea_orm(...)]
    lines_before = content[:type_start].split("\n")
    derive_line_idx = None
    sea_orm_line_idx = None
    derive_match = None

    # Check the last few lines before the type
    for i in range(len(lines_before) - 1, max(-1, len(lines_before) - 5), -1):
        line = lines_before[i]
        if re.search(r"#\[derive\([^)]+\)\]", line):
            derive_line_idx = i
            derive_match = re.search(r"#\[derive\(([^)]+)\)\]", line)
        if re.search(r"#\[sea_orm\([^)]+\)\]", line):
            sea_orm_line_idx = i

    # Build the lines list for easier manipulation
    lines = content.split("\n")
    type_line_idx = content[:type_start].count("\n")

    if derive_match and derive_line_idx:
        # Found existing derive
        derive_line = lines[derive_line_idx]
        existing_derives = derive_match.group(1)

        # Check if derive already exists
        if derives in existing_derives:
            print(f"Derive {derives} already exists for {type_name}")
            return

        # Add the new derive
        new_derives = f"{existing_derives}, {derives}"
        new_derive_line = re.sub(
            r"#\[derive\([^)]+\)\]", f"#[derive({new_derives})]", derive_line
        )
        lines[derive_line_idx] = new_derive_line

        # Add attributes if provided (after sea_orm attribute, before enum)
        if attributes:
            # Check if attributes already exist
            attr_exists = False
            for i in range(max(0, type_line_idx - 3), type_line_idx):
                if attributes in lines[i]:
                    attr_exists = True
                    break

            if not attr_exists:
                # Insert after sea_orm line if it exists, otherwise before type
                if sea_orm_line_idx is not None:
                    lines.insert(sea_orm_line_idx + 1, attributes)
                else:
                    lines.insert(type_line_idx, attributes)
    else:
        # No derive found, add it before the type definition
        # But we need to place it before sea_orm if that exists
        if sea_orm_line_idx is not None:
            # Insert derive before sea_orm
            lines.insert(sea_orm_line_idx, f"#[derive({derives})]")
            if attributes:
                # Insert attributes after sea_orm
                lines.insert(sea_orm_line_idx + 2, attributes)
        else:
            # Insert before type
            new_derive = f"#[derive({derives})]"
            if attributes:
                lines.insert(type_line_idx, attributes)
                lines.insert(type_line_idx + 1, new_derive)
            else:
                lines.insert(type_line_idx, new_derive)

    # Add utoipa import if ToSchema is being added and it's not already imported
    if "ToSchema" in derives and "use utoipa::ToSchema;" not in content:
        # Find the last use statement
        last_use_idx = None
        for i, line in enumerate(lines):
            if line.strip().startswith("use ") and line.strip().endswith(";"):
                last_use_idx = i

        # Insert after the last use statement
        if last_use_idx is not None:
            lines.insert(last_use_idx + 1, "use utoipa::ToSchema;")
        else:
            # Insert after the module comment
            for i, line in enumerate(lines):
                if line.strip().startswith("//!"):
                    lines.insert(i + 1, "")
                    lines.insert(i + 2, "use utoipa::ToSchema;")
                    break

    # Write back
    file.write_text("\n".join(lines))
    print(f"Successfully applied derives to {type_name} in {file_path}")


if __name__ == "__main__":
    if len(sys.argv) < 4:
        print("Usage:")
        print(
            "  Type-level: python3 apply_derives.py <file_path> <type_name> <derives> [attributes]"
        )
        print(
            "  Field-level: python3 apply_derives.py <file_path> <type_name> --field <field_name> <attribute>"
        )
        print()
        print("Examples:")
        print(
            '  python3 apply_derives.py crates/entities/src/sea_orm_active_enums.rs OauthProvider "ToSchema" "#[serde(rename_all = \\"lowercase\\")]"'
        )
        print(
            '  python3 apply_derives.py crates/entities/src/user.rs User --field password "#[serde(skip)]"'
        )
        sys.exit(1)

    file_path = sys.argv[1]
    type_name = sys.argv[2]

    # Check if this is a field-level operation
    if len(sys.argv) >= 6 and sys.argv[3] == "--field":
        field_name = sys.argv[4]
        attribute = sys.argv[5]
        apply_field_attribute(file_path, type_name, field_name, attribute)
    else:
        # Type-level operation
        derives = sys.argv[3]
        attributes = sys.argv[4] if len(sys.argv) > 4 else None
        apply_derives(file_path, type_name, derives, attributes)
