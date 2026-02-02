import sys
import json

def main():
    if len(sys.argv) < 2:
        sys.exit(1)
        
    try:
        with open(sys.argv[1], 'r') as f:
            data = json.load(f)
        
        # Validasi Sederhana: Pastikan root adalah Program
        if data.get("type") != "Program":
            print(json.dumps({"error": "Invalid AST Root: Expected Program"}))
            sys.exit(1)
            
        # Tambahkan metadata validasi
        if "metadata" not in data:
            data["metadata"] = {}
        data["metadata"]["sema_validated"] = True
        
        # Output JSON ke stdout
        print(json.dumps(data, indent=2))
        
    except Exception as e:
        sys.stderr.write(f"Sema Error: {str(e)}
")
        sys.exit(1)

if __name__ == "__main__":
    main()
