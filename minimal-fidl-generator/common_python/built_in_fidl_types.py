from dataclasses import dataclass
from enum import IntEnum, Enum
from typing import ClassVar
import struct
from abc import ABC
from enum import EnumMeta
from typing import Self




@dataclass(frozen=True)
class BinarySerdeIntEnum(IntEnum):
    def __init__(self, value: int, size: int, struct_format: str, lower_range_limit: int, upper_range_limit: int):
        super().__init__() 
        self._size = size
        type(self)._size = size # So class method has access to _size
        type(self)._struct_format = struct_format
        self._struct_format = struct_format
        self._lower_range_limit = lower_range_limit
        self._upper_range_limit = upper_range_limit
        
        if int(self.value) < self._lower_range_limit or int(self.value) > self._upper_range_limit:
            raise ValueError(f"{self.__class__.__name__}  must be between {self._lower_range_limit} and {self._upper_range_limit}")

    def __str__(self):
        return f"({type(self).__name__}.{self.name}: {self.value}, Size: {self._size})"
    
    
    def __int__(self):
        return self.value
    
    def __bytes__(self) -> bytes:
        array = struct.pack(self._struct_format, self.value)
        return array
    
    @classmethod
    def size(cls):
        size = cls._size
        return size

    @classmethod
    def from_bytes(cls, input: list[bytes]):
        if len(input) != cls._size:
            raise ValueError(f"{cls.__name__} can only be initialized from {cls._size} bytes")
        input = struct.unpack(cls._struct_format, input)[0] 
        return cls(input)

@dataclass(frozen=True)
class u8IntEnum(BinarySerdeIntEnum):

    def __init__(self, value: int):
        size = 1
        struct_format = "<B"
        lower_range_limit = 0
        upper_range_limit = 255
        super().__init__(value, size, struct_format, lower_range_limit, upper_range_limit)

@dataclass(frozen=True)
class u16IntEnum(BinarySerdeIntEnum):

    def __init__(self, value: int):
        # value is required in the function signature but is consumed by the IntEnum constructor that runs eventually.
        size: int = 2
        struct_format: str = "<H"
        lower_range_limit: int = 0 
        upper_range_limit: int = 65535
        super().__init__(value, size, struct_format, lower_range_limit, upper_range_limit)

@dataclass(frozen=True)
class u32IntEnum(BinarySerdeIntEnum):

    def __init__(self, value: int):
        # value is required in the function signature but is consumed by the IntEnum constructor that runs eventually.
        struct_format: str = "<I"
        size: int = 4
        lower_range_limit: int = 0
        upper_range_limit: int = 4294967295 
        super().__init__(value, size, struct_format, lower_range_limit, upper_range_limit)



@dataclass(frozen=True)
class u64IntEnum(BinarySerdeIntEnum):

    def __init__(self, value: int):
        # value is required in the function signature but is consumed by the IntEnum constructor that runs eventually.
        struct_format: str = "<Q"
        size: int = 8
        lower_range_limit: int = 0
        upper_range_limit: int = (2**64)-1 
        super().__init__(value, size, struct_format, lower_range_limit, upper_range_limit)


@dataclass(frozen=True)
class Boolean():
    value: bool
    _struct_format: ClassVar[str] = "?"
    _size: ClassVar[int] = 1
    
    def __repr__(self) -> str:
        return str(self.value)
    
    def __bytes__(self) -> bytes:
        array = struct.pack(self._struct_format, self.value)
        return array

    def __post_init__(self):
        if not isinstance(self.value, bool):
            raise TypeError(f"{self.__class__.__name__} '{type(self.value).__name__}: {self.value}' is not a valid input for {type(self).__name__}")
    
    @classmethod
    def from_bytes(cls, input: list[bytes]):
        if len(input) != cls._size:
            raise ValueError(f"{cls.__name__} can only be initialized from {cls._size} bytes")
        input = struct.unpack(cls._struct_format, input)[0] 
        return cls(input)
    


@dataclass(frozen=True)
class BaseFloatingPointPrimitive(ABC):
    value: float
    _struct_format: ClassVar[str]
    _size: ClassVar[int]
    
    def __str__(self) -> str:
        return str(self.value)
    
        
    def __float__(self):
        return self.value
    
    def __bytes__(self) -> bytes:
        array = struct.pack(self._struct_format, self.value)
        return array
    

    def __post_init__(self):
        # TODO: Currently no validation of size of floats because not entirely sure how. 
        if not isinstance(self.value, float):
            raise TypeError(f"{self.__class__.__name__} '{type(self.value).__name__}: {self.value}' is not a valid input for {type(self).__name__}")
    
    @classmethod
    def size(cls):
        return cls._size

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
    
    def __str__(self) -> str:
        return str(self.value)
        
    def __int__(self):
        return self.value
    
    def __bytes__(self) -> bytes:
        array = struct.pack(self._struct_format, self.value)
        return array

    def __post_init__(self):
        if not isinstance(self.value, int):
            raise TypeError(f"{self.__class__.__name__} '{self.value}' is not a valid input for {type(self).__name__} ")
        if int(self.value) < self._lower_range_limit or int(self.value) > self._upper_range_limit:
            raise ValueError(f"{self.__class__.__name__}  must be between {self._lower_range_limit} and {self._upper_range_limit}")
    
    @classmethod
    def size(cls):
        return cls._size

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

@dataclass(frozen=True)
class BinarySerdeStruct(ABC):
    
    def __bytes__(self) -> bytes:
        result = bytes()
        for field in self.__dataclass_fields__:
            result = result + bytes(self.__getattribute__(field))
        return result

    @classmethod
    def from_bytes(cls, input: list[bytes]):
        position = 0
        field_value_map = {}
        for field in cls.__dataclass_fields__:
            field_type = cls.__dataclass_fields__[field].type
            size = field_type.size()
            if size != None:
                new_obj = field_type.from_bytes(input[position:position+size])
            else:
                (new_obj, size) = field_type.dynamic_from_bytes(input[position:]) # Assumes unsized elements are at the end of the struct
            position += size
            field_value_map[field] = new_obj
        return cls(**field_value_map)
    
    @classmethod
    def size(cls):
        size = cls._size
        return size

    def __post_init__(self):
        sized: bool = True
        size = 0
        for field in self.__dataclass_fields__:
            attribute = self.__getattribute__(field)
            typ = self.__dataclass_fields__[field].type
            if not isinstance(attribute, typ): 
                raise ValueError(f"Struct: '{self.__class__.__name__}', Field: '{field}' must be of type '{typ.__name__}' not '{type(attribute).__name__}'")
            if typ.size() == None:
                sized = False
            else:
                size += typ.size()
        if sized:
            type(self)._size = size
        else: 
            type(self)._size = None


@dataclass(frozen=True)
class String():
    ''' Zero Char terminated String'''
    value: str

    def __bytes__(self) -> bytes:
        result = bytes(self.value + "\0", encoding="ASCII")
        return result
    
    @classmethod
    def dynamic_from_bytes(cls, input: bytes) -> tuple[int, bytes]:
        return cls._from_bytes(input)

    @classmethod
    def _from_bytes(cls, input: bytes) -> tuple[Self, int]:
        actual_input = None
        for index, char in enumerate(input):
            if char == 0:
                actual_input = input[0:index]
                break # We break because we can get passed more data than the actual string since we don't know yet where the null char is. 
        if actual_input == None:
            raise ValueError("Not a null terminated string.")
        return (cls(str(actual_input, encoding="ASCII")), len(actual_input)+1)

    @classmethod
    def from_bytes(cls, input: bytes):
        return cls._from_bytes(input)[1]
    
    @classmethod
    def size(cls):
        size = None # None indicates it's dynamically sized and the callee needs to use 'dynamic_from_bytes'
        return size
    
    def __post_init__(self):
        if not isinstance(self.value, str):
            raise ValueError("String input must be a string")
        for index, char in enumerate(self.value):
            if ord(char) == 0:
                # CAREFUL: The null characters could be from being passed too large an input and getting other data.
                raise ValueError("String input cannot contain a null character")

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




############################################################################################
# The following is test code, this should be moved out with the common fidl types being    #
# tested properly in a package, this is currently a WIP temporary measure.                 #
############################################################################################


if __name__ == "__main__":
    class TestEnum(u8IntEnum):
        THING = 2
        THING3 = 3

    class TestEnum2(u16IntEnum):
        THINGY = 3

    @dataclass(frozen=True)
    class ThingStruct(BinarySerdeStruct):
        some_value: u8
        some_value2: f32

    @dataclass(frozen=True)
    class ThingStruct2(BinarySerdeStruct):
        some_value: i64
        some_value2: ThingStruct

    @dataclass(frozen=True)
    class ThingStruct3(BinarySerdeStruct):
        some_value: i64
        some_value2: TestEnum

    @dataclass(frozen=True)
    class ThingStruct4(BinarySerdeStruct):
        some_value: i64
        some_value2: String
        some_value3: u32

    

    x = UInt8(20)
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

    x = Boolean(True)
    print(x)
    y = bytes(x)
    print(y.hex())
    z = Boolean.from_bytes(y)
    print(z)

    f = TestEnum.THING
    print(f)
    print(f == 2)
    z = bytes(f)
    print(z)
    print(f.from_bytes(z))

    f2 = TestEnum2.THINGY
    z = bytes(f2)
    print(z)
    print(f2.from_bytes(z))

    f = ThingStruct(u8(2), f32(0.5))
    z = bytes(f)
    print(z, len(z))

    f2 = ThingStruct.from_bytes(z)
    print(f"Sizeof: {f2.size()}")

    print(f2)


    f = ThingStruct2(i64(20), ThingStruct(u8(2), f32(0.5)))
    print(f"Sizeof: {f.size()}")
    print(f)    
    z = bytes(f)
    print(z, len(z))

    f2 = ThingStruct2.from_bytes(z)
    print(f2)


    f = ThingStruct3(i64(20), TestEnum.THING)
    print(f"Sizeof: {f.size()}")
    print(f)    
    z = bytes(f)
    print(z, len(z))

    f2 = ThingStruct3.from_bytes(z)
    print(f2, f2.some_value2)

    f = String("teataogaiwg")
    print(f)
    f2 = bytes(f)
    print(f2)
    f3 = String.from_bytes(f2)
    print(f3)

    f = ThingStruct4(i64(20), String("Whatever"), u32(5))
    print(f)
    f2 = bytes(f)
    print(f2, len(f2))
    f3 = ThingStruct4.from_bytes(f2)
    print(f3)