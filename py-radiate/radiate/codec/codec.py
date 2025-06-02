import abc
from radiate.radiate import PyAnyCodec


class CodecBase(abc.ABC):

    pass

    
class AnyCodec(CodecBase):
    """
    AnyCodec is a base class for codecs that can handle any type of data.
    It is used to encode and decode chromosomes into a generic format.
    """

    def __init__(self):
        """
        Initialize the AnyCodec with a codec.
        :param codec: The codec to use for encoding and decoding.
        """

        self.codec = PyAnyCodec()

    def test(self, value):
        """
        Test if the value can be encoded and decoded correctly.
        :param value: The value to test.
        :return: True if the value can be encoded and decoded correctly, False otherwise.
        """
        return self.codec.test(value)
