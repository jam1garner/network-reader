use std::io::{self, BufReader, BufWriter, SeekFrom, prelude::*};
use std::net::{ToSocketAddrs, TcpStream, TcpListener};
use std::convert::TryInto;

const OPERATION_READ: u8 = 0xFF;
const OPERATION_SEEK: u8 = 0xFE;

const SEEK_FROM_START: u8 = 0;
const SEEK_FROM_END: u8 = 1;
const SEEK_FROM_CURRENT: u8 = 2;

const RESULT_OK: u8 = 0;
const RESULT_ERR: u8 = 1;

pub struct Networked<R: Read + Seek> {
    listener: TcpListener,
    reader: R,
}

impl<R: Read + Seek> Networked<R> {
    pub fn new<S: ToSocketAddrs>(reader: R, socket: S) -> io::Result<Self> {
        Ok(Self {
            reader,
            listener: TcpListener::bind(socket)?
        })
    }
    
    pub fn new_buffered<S: ToSocketAddrs>(reader: R, socket: S) -> io::Result<Networked<BufReader<R>>> {
        Ok(Networked {
            reader: BufReader::new(reader),
            listener: TcpListener::bind(socket)?
        })
    }
    
    pub fn listen(mut self) -> io::Result<()> {
        for connection in self.listener.incoming() {
            let mut connection = connection?;
            let mut buf = [0u8];
            while connection.read_exact(&mut buf).is_ok() {
                match buf[0] {
                    OPERATION_SEEK => {
                        let mut buf = [0u8; 9];
                        let pos = match connection.read_exact(&mut buf) {
                            Ok(_) => {
                                let offset = i64::from_be_bytes(buf[1..].try_into().unwrap());
                                match buf[0] {
                                    0 => SeekFrom::Start(offset as u64),
                                    1 => SeekFrom::End(offset),
                                    2 => SeekFrom::Current(offset),
                                    _ => continue
                                }
                            },
                            Err(_) => continue,
                        };

                        match self.reader.seek(pos) {
                            Ok(ret) => {
                                connection.write_all(&[RESULT_OK])?;
                                connection.write_all(&u64::to_be_bytes(ret))?;
                            }
                            Err(_) => {
                                connection.write_all(&[RESULT_ERR])?;
                            }
                        }
                        connection.flush()?;
                    }
                    OPERATION_READ => {
                        let mut buf = [0u8; 8];
                        let amount = match connection.read_exact(&mut buf) {
                            Ok(_) => u64::from_be_bytes(buf),
                            Err(_) => continue,
                        };
                        
                        let mut writer = BufWriter::new(&mut connection);
                        let reader = &mut self.reader;
                        let size = io::copy(&mut reader.take(amount), &mut writer)?;

                        io::copy(&mut io::repeat(0).take(amount - size), &mut writer)?;
                        writer.write_all(&size.to_be_bytes())?;
                        writer.flush()?;
                    }
                    _ => continue
                }
            }
        }
        Ok(())
    }
}

pub struct NetworkReader(TcpStream);

impl NetworkReader {
    pub fn new<Addr: ToSocketAddrs>(addr: Addr) -> io::Result<Self> {
        TcpStream::connect(addr).map(Self)
    }
}

impl Seek for NetworkReader {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.0.write_all(&[OPERATION_SEEK])?;
        self.0.write_all(&[match pos {
            SeekFrom::Start(_) => SEEK_FROM_START,
            SeekFrom::End(_) => SEEK_FROM_END,
            SeekFrom::Current(_) => SEEK_FROM_CURRENT,
        }])?;
        self.0.write_all(&match pos {
            SeekFrom::Start(offset) => offset.to_be_bytes(),
            SeekFrom::End(offset) | SeekFrom::Current(offset) => offset.to_be_bytes(),
        })?;
        self.0.flush()?;

        let mut result = [0u8];
        self.0.read_exact(&mut result)?;

        if result == [RESULT_OK] {
            let mut val = [0u8; 8];
            self.0.read_exact(&mut val)?;
            
            Ok(u64::from_be_bytes(val))
        } else {
            Err(io::Error::new(io::ErrorKind::Other, "server returned error"))
        }
    }
}

impl Read for NetworkReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.write_all(&[OPERATION_READ])?;
        self.0.write_all(&(buf.len() as u64).to_be_bytes())?;
        
        let mut buf = [0u8; 8];
        self.0.read_exact(&mut buf)?;

        Ok(u64::from_be_bytes(buf) as usize)
    }
}
