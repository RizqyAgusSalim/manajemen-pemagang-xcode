# Tahap 1: Build aplikasi
FROM rust:1.80 as builder
WORKDIR /usr/src/app
COPY . .
RUN cargo build --release

# Tahap 2: Jalankan aplikasi menggunakan container Ubuntu yang lebih ringan
FROM ubuntu:22.04
WORKDIR /usr/src/app

# Install dependency tambahan jika diperlukan (seperti OpenSSL untuk JWT/Database)
RUN apt-get update && apt-get install -y ca-certificates libssl-dev && rm -rf /var/lib/apt/lists/*

# Copy file binary dan aset
COPY --from=builder /usr/src/app/target/release/intern_management /usr/src/app/intern_management
# Copy folder static (jika ada) - abaikan jika tidak ada
# COPY --from=builder /usr/src/app/static /usr/src/app/static
# Buat folder uploads
RUN mkdir -p /usr/src/app/uploads

# Buka port yang digunakan aplikasi
EXPOSE 3000

# Jalankan server
CMD ["./intern_management"]
