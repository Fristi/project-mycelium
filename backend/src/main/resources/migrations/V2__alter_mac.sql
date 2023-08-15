ALTER TABLE stations alter column mac_addr TYPE varchar;
ALTER TABLE stations ADD CONSTRAINT unique_mac UNIQUE (mac_addr);