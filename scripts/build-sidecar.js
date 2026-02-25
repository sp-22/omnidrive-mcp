import { execSync } from 'child_process';
import { copyFileSync, mkdirSync, existsSync, writeFileSync } from 'fs';
import { join } from 'path';

// Get target triple from rustc
const rustcInfo = execSync('rustc -vV').toString();
const targetMatch = rustcInfo.match(/host: (.+)/);
if (!targetMatch) {
    console.error('Could not determine Rust target triple');
    process.exit(1);
}
const targetTriple = targetMatch[1];
const isWindows = targetTriple.includes('windows');
const sidecarFileName = `omnidrive_server-${targetTriple}${isWindows ? '.exe' : ''}`;
const builtBinaryName = `omnidrive_server${isWindows ? '.exe' : ''}`;

console.log(`Building sidecar for target: ${targetTriple}`);

// Create binaries directory if not exists
const binDir = join(process.cwd(), 'src-tauri', 'binaries');
if (!existsSync(binDir)) {
    mkdirSync(binDir, { recursive: true });
}

// Create a dummy file FIRST to satisfy tauri build.rs
const destBinaryPath = join(binDir, sidecarFileName);
if (!existsSync(destBinaryPath)) {
    console.log('Creating dummy binary for tauri-build...');
    writeFileSync(destBinaryPath, '');
}

// Build the release binary
execSync('cargo build --release --bin omnidrive_server', {
    stdio: 'inherit',
    cwd: join(process.cwd(), 'src-tauri')
});

// Copy binary to correct location with target appendeed
const srcBinaryPath = join(process.cwd(), 'src-tauri', 'target', 'release', builtBinaryName);

console.log(`Copying sidecar from ${srcBinaryPath} to ${destBinaryPath}`);
copyFileSync(srcBinaryPath, destBinaryPath);
console.log('Sidecar bundled successfully!');
