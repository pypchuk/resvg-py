from io import BytesIO
from os.path import join
from tempfile import TemporaryDirectory

from glyphcast.formats import Format
from glyphcast.utils import execute

from cairosvg import svg2pdf

class Converter:

    def __init__(self, source_format, to_format):
        self.source_format = source_format
        self.to_format = to_format

    @property
    def conversion_fn(self):
        """ Given a source format and a destination format ("to-format"),
            return the appropriate conversion method if available, or None

            The signature of conversion functions is as follows:

            conversion_function(input_data: bytes) -> converted_data: bytes

            where input_data is the file data to be converted, and converted_data
            is the input data converted to the destination format
        """
        conversion = (self.source_format, self.to_format)
        return {
            (Format.SVG, Format.PDF): self.svg_to_pdf,
            (Format.DOCX, Format.PDF): self.docx_to_pdf
        }.get(conversion)

    def converted_mimetype(self):
        {
            Format.PDF: "application/pdf"
        }.get(self.to_format)


    def convert(self, bytes_):
        """ If no conversion method is available, raise an UnsupportedConversionException
            otherwise attempt to perform the conversion.
        """

        if not self.conversion_fn:
            message = f"The conversion {self.source_format} -> {self.to_format} is not supported"
            raise UnsupportedConversionException(message)

        return self.conversion_fn(bytes_)


    @staticmethod
    def svg_to_pdf(svg_text):
        pdf_buffer = BytesIO()
        buffer_size = 0
        decoded = svg_text.decode("latin1")
        svg2pdf(bytestring=decoded, write_to=pdf_buffer)
        buffer_size += pdf_buffer.tell()
        # Reset the pdf_buffer stream position to 0
        pdf_buffer.seek(0)
        return pdf_buffer, buffer_size


    @staticmethod
    def docx_to_pdf(docx):
        """ Convert files in docx format to PDF using LibreOffice lowriter
        """
        pdf_buffer = BytesIO()
        buffer_size = 0
        # Create a temporary directory that is unlinked as soon as we exit the
        # tempdir context
        with TemporaryDirectory() as tempdir:
            docx_path = join(tempdir, "document.docx")
            # Write the input data to a file in the temp directory
            with open(docx_path, "wb") as source_file:
                source_file.write(docx)
            run_lowriter = ["lowriter", "--invisible", "--convert-to", "pdf", f"{docx_path}", "--outdir", tempdir]
            #run_unoconv = ["/usr/bin/python3", "/usr/bin/unoconv", "-f", "pdf", f"{docx_path}"]
            # Run LibreOffice lowriter against the tempdir input file
            execute(run_lowriter, raise_error=True)
            pdf_path = join(tempdir, "document.pdf")
            with open(pdf_path, "rb") as outfile:
                # Write the file output by lowriter to pdf_buffer
                buffer_size += pdf_buffer.write(outfile.read())

        # Reset the pdf_buffer stream position to 0 otherwise there might be unexpected behavior in callers --
        # for example, Flask.send_file will only send data after the current stream position, so calling Flask.send_file
        # immediately on the return value of this method will send an empty byte stream
        pdf_buffer.seek(0)
        return pdf_buffer, buffer_size

    @staticmethod
    def conversion_type(from_: str, to: str) -> (Format, Format):
        if not (from_ and to):
            return (Format.UNKNOWN, Format.UNKNOWN)

        from_upper = from_.upper()
        to_upper = to.upper()

        supported_format = lambda format_: format_ in dir(Format)

        if not (supported_format(from_upper) and supported_format(to_upper)):
            return (Format.UNKNOWN, Format.UNKNOWN)

        return (Format[from_upper], Format[to_upper])




class UnsupportedConversionException(Exception):

    def __init__(self, message):
        super().__init__(message)
