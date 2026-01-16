#!/usr/bin/env python3
"""Script to add Mv routing to main.rs"""

import sys

def add_mv_routing():
    main_rs_path = "src/main.rs"

    # Read the file
    with open(main_rs_path, 'r') as f:
        lines = f.readlines()

    # Find the Uninstall block and add Mv after it
    new_lines = []
    i = 0
    while i < len(lines):
        new_lines.append(lines[i])

        # Look for the end of Uninstall block
        if 'IdbCommands::Uninstall { bundle_id, udid } => {' in lines[i]:
            # Add current line and the next line (the run call)
            new_lines.append(lines[i + 1])
            i += 2
            # Add the closing brace
            new_lines.append(lines[i])

            # Now add the Mv routing
            mv_routing = '''            IdbCommands::Mv {
                src_paths,
                dst_path,
                bundle_id,
                root,
                udid,
            } => {
                cli::idb::mv::run(src_paths, dst_path, bundle_id, root, udid).await?;
            }
'''
            new_lines.append(mv_routing)
            i += 1
            continue

        i += 1

    # Write back
    with open(main_rs_path, 'w') as f:
        f.writelines(new_lines)

    print("✅ Successfully added Mv routing to main.rs")

if __name__ == '__main__':
    try:
        add_mv_routing()
    except Exception as e:
        print(f"❌ Error: {e}", file=sys.stderr)
        sys.exit(1)
