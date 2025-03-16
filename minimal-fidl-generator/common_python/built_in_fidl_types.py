from dataclasses import dataclass
from typing import ClassVar
import struct
from abc import ABC

@dataclass(frozen=True)
class BaseFloatingPointPrimitive(ABC):
    value: float
    _struct_format: ClassVar[str]
    _size: ClassVar[int]
    
    def __repr__(self) -> str:
        return str(self.value)
    
    def __bytes__(self) -> bytes:
        array = struct.pack(self._struct_format, self.value)
        return array

    def __post_init__(self):
        # TODO: Currently no validation of size of floats because not entirely sure how. 
        if not isinstance(self.value, float):
            raise TypeError(f"{self.__class__.__name__} '{self.value}' is not a valid input for {type(self).__name__}")
    
    @classmethod
    def from_bytes(cls, input: list[bytes]):
        if len(input) != cls._size:
            raise ValueError(f"{cls.__name__} can only be initialized from {cls._size} bytes")
        input = struct.unpack(cls._struct_format, input)[0] 
        return cls(input)
    


@dataclass(frozen=True)
class BaseIntegerPrimitive(ABC):
    value: int
    _struct_format: ClassVar[str]
    _size: ClassVar[int]
    _lower_range_limit: ClassVar[int]
    _upper_range_limit: ClassVar[int]
    
    def __repr__(self) -> str:
        return str(self.value)
    
    def __bytes__(self) -> bytes:
        array = struct.pack(self._struct_format, self.value)
        return array

    def __post_init__(self):
        if not isinstance(self.value, int):
            raise TypeError(f"{self.__class__.__name__} '{self.value}' is not a valid input for {type(self).__name__} ")
        if int(self.value) < self._lower_range_limit or int(self.value) > self._upper_range_limit:
            raise ValueError(f"{self.__class__.__name__}  must be between {self._lower_range_limit} and {self._upper_range_limit}")
    
    @classmethod
    def from_bytes(cls, input: list[bytes]):
        if len(input) != cls._size:
            raise ValueError(f"{cls.__name__} can only be initialized from {cls._size} bytes")
        input = struct.unpack(cls._struct_format, input)[0] 
        return cls(input)
    
    

@dataclass(frozen=True)
class u8(BaseIntegerPrimitive):
    _struct_format: ClassVar[str] = "<B"
    _size: ClassVar[int] = 1
    _lower_range_limit: ClassVar[int] = 0 
    _upper_range_limit: ClassVar[int] = 255
    


@dataclass(frozen=True)
class i8(BaseIntegerPrimitive):
    _struct_format: ClassVar[str] = "<b"
    _size: ClassVar[int] = 1
    _lower_range_limit: ClassVar[int] = -127 
    _upper_range_limit: ClassVar[int] = 128


@dataclass(frozen=True)
class u16(BaseIntegerPrimitive):
    _struct_format: ClassVar[str] = "<H"
    _size: ClassVar[int] = 2
    _lower_range_limit: ClassVar[int] = 0 
    _upper_range_limit: ClassVar[int] = 65535


@dataclass(frozen=True)
class i16(BaseIntegerPrimitive):
    _struct_format: ClassVar[str] = "<h"
    _size: ClassVar[int] = 2
    _lower_range_limit: ClassVar[int] = -32768
    _upper_range_limit: ClassVar[int] = 32767 

@dataclass(frozen=True)
class u32(BaseIntegerPrimitive):
    _struct_format: ClassVar[str] = "<I"
    _size: ClassVar[int] = 4
    _lower_range_limit: ClassVar[int] = 0
    _upper_range_limit: ClassVar[int] = 4294967295 

@dataclass(frozen=True)
class i32(BaseIntegerPrimitive):
    _struct_format: ClassVar[str] = "<i"
    _size: ClassVar[int] = 4
    _lower_range_limit: ClassVar[int] = -2147483648
    _upper_range_limit: ClassVar[int] = 2147483647 

@dataclass(frozen=True)
class u64(BaseIntegerPrimitive):
    _struct_format: ClassVar[str] = "<Q"
    _size: ClassVar[int] = 8
    _lower_range_limit: ClassVar[int] = 0
    _upper_range_limit: ClassVar[int] = (2**64)-1 

@dataclass(frozen=True)
class i64(BaseIntegerPrimitive):
    _struct_format: ClassVar[str] = "<q"
    _size: ClassVar[int] = 8
    _lower_range_limit: ClassVar[int] = -9223372036854775808
    _upper_range_limit: ClassVar[int] = 9223372036854775807

@dataclass(frozen=True)
class f32(BaseFloatingPointPrimitive):
    _struct_format: ClassVar[str] = "<f"
    _size: ClassVar[int] = 4

@dataclass(frozen=True)
class f64(BaseFloatingPointPrimitive):
    _struct_format: ClassVar[str] = "<d"
    _size: ClassVar[int] = 8



class UInt8(u8):
    '''Type stub to match FIDL'''
    
class Int8(i8):
    '''Type stub to match FIDL'''

class UInt16(u16):
    '''Type stub to match FIDL'''

class Int16 (i16):
    '''Type stub to match FIDL'''

class UInt32(u32):
    '''Type stub to match FIDL'''

class Int32(i32):
    '''Type stub to match FIDL'''

class UInt64(u64):
    '''Type stub to match FIDL'''

class Int64(i64):
    '''Type stub to match FIDL'''

class Float(f32):
    '''Type stub to match FIDL'''

class Double(f64):
    '''Type stub to match FIDL'''



if __name__ == "__main__":
    x = UInt8(45)
    y = u16(20)
    print(x, y, x == y)
    z = bytes(x)
    print(z)
    print(UInt8.from_bytes(z))

    print("----------")
    x = u16(32000)
    y = u16(100)
    print(x, y, x == y)
    z = bytes(x)
    print(z.hex())

    print("-------")
    x = f32(20.)
    y = bytes(x)
    print(x, y.hex(" ", 1))
    z = f32.from_bytes(y)
    print(z)
