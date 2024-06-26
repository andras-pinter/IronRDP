// <auto-generated/> by Diplomat

#pragma warning disable 0105
using System;
using System.Runtime.InteropServices;

using Devolutions.IronRdp.Diplomat;
#pragma warning restore 0105

namespace Devolutions.IronRdp;

#nullable enable

public partial class BytesSlice: IDisposable
{
    private unsafe Raw.BytesSlice* _inner;

    public nuint Size
    {
        get
        {
            return GetSize();
        }
    }

    /// <summary>
    /// Creates a managed <c>BytesSlice</c> from a raw handle.
    /// </summary>
    /// <remarks>
    /// Safety: you should not build two managed objects using the same raw handle (may causes use-after-free and double-free).
    /// <br/>
    /// This constructor assumes the raw struct is allocated on Rust side.
    /// If implemented, the custom Drop implementation on Rust side WILL run on destruction.
    /// </remarks>
    public unsafe BytesSlice(Raw.BytesSlice* handle)
    {
        _inner = handle;
    }

    public nuint GetSize()
    {
        unsafe
        {
            if (_inner == null)
            {
                throw new ObjectDisposedException("BytesSlice");
            }
            nuint retVal = Raw.BytesSlice.GetSize(_inner);
            return retVal;
        }
    }

    /// <exception cref="IronRdpException"></exception>
    public void Fill(byte[] buffer)
    {
        unsafe
        {
            if (_inner == null)
            {
                throw new ObjectDisposedException("BytesSlice");
            }
            nuint bufferLength = (nuint)buffer.Length;
            fixed (byte* bufferPtr = buffer)
            {
                Raw.UtilsFfiResultVoidBoxIronRdpError result = Raw.BytesSlice.Fill(_inner, bufferPtr, bufferLength);
                if (!result.isOk)
                {
                    throw new IronRdpException(new IronRdpError(result.Err));
                }
            }
        }
    }

    /// <summary>
    /// Returns the underlying raw handle.
    /// </summary>
    public unsafe Raw.BytesSlice* AsFFI()
    {
        return _inner;
    }

    /// <summary>
    /// Destroys the underlying object immediately.
    /// </summary>
    public void Dispose()
    {
        unsafe
        {
            if (_inner == null)
            {
                return;
            }

            Raw.BytesSlice.Destroy(_inner);
            _inner = null;

            GC.SuppressFinalize(this);
        }
    }

    ~BytesSlice()
    {
        Dispose();
    }
}
