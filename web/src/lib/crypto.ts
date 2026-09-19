export interface WhatsRookConfig {
    phone: string;
    business: boolean;
    auth: 'qr' | 'pair';
    client: 'default' | 'android' | 'ios';
    db: string;
    autoupdate: boolean;
    verbose: boolean;
}

const RAW_KEY = new TextEncoder().encode("ThruqeCustomWABotMasterKey_2026!");

async function getCryptoKey(): Promise<CryptoKey> {
    return await crypto.subtle.importKey(
        "raw",
        RAW_KEY,
        { name: "AES-GCM" },
        false,
        ["encrypt"]
    );
}

async function compress(str: string): Promise<Uint8Array> {
    const stream = new Blob([str]).stream().pipeThrough(new CompressionStream("deflate-raw"));
    const buffer = await new Response(stream).arrayBuffer();
    return new Uint8Array(buffer);
}

function toBase64RawUrl(bytes: Uint8Array): string {
    let binary = "";
    for (let i = 0; i < bytes.byteLength; i++) {
        binary += String.fromCharCode(bytes[i]);
    }
    return btoa(binary).replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/, "");
}

export async function generateSessionToken(config: WhatsRookConfig): Promise<string> {
    const jsonStr = JSON.stringify(config);
    const compressed = await compress(jsonStr);

    const key = await getCryptoKey();
    const nonce = crypto.getRandomValues(new Uint8Array(12));

    const encryptedBuffer = await crypto.subtle.encrypt(
        { name: "AES-GCM", iv: nonce },
        key,
        compressed
    );

    const ciphertext = new Uint8Array(encryptedBuffer);
    const combined = new Uint8Array(nonce.length + ciphertext.length);
    combined.set(nonce, 0);
    combined.set(ciphertext, nonce.length);

    return `SE_ID:${toBase64RawUrl(combined)}`;
}