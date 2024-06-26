// <auto-generated/> by Diplomat

#pragma warning disable 0105
using System;
using System.Runtime.InteropServices;

using Devolutions.IronRdp.Diplomat;
#pragma warning restore 0105

namespace Devolutions.IronRdp;

#nullable enable

public partial class ActiveStage: IDisposable
{
    private unsafe Raw.ActiveStage* _inner;

    /// <summary>
    /// Creates a managed <c>ActiveStage</c> from a raw handle.
    /// </summary>
    /// <remarks>
    /// Safety: you should not build two managed objects using the same raw handle (may causes use-after-free and double-free).
    /// <br/>
    /// This constructor assumes the raw struct is allocated on Rust side.
    /// If implemented, the custom Drop implementation on Rust side WILL run on destruction.
    /// </remarks>
    public unsafe ActiveStage(Raw.ActiveStage* handle)
    {
        _inner = handle;
    }

    /// <exception cref="IronRdpException"></exception>
    /// <returns>
    /// A <c>ActiveStage</c> allocated on Rust side.
    /// </returns>
    public static ActiveStage New(ConnectionResult connectionResult)
    {
        unsafe
        {
            Raw.ConnectionResult* connectionResultRaw;
            connectionResultRaw = connectionResult.AsFFI();
            if (connectionResultRaw == null)
            {
                throw new ObjectDisposedException("ConnectionResult");
            }
            Raw.SessionFfiResultBoxActiveStageBoxIronRdpError result = Raw.ActiveStage.New(connectionResultRaw);
            if (!result.isOk)
            {
                throw new IronRdpException(new IronRdpError(result.Err));
            }
            Raw.ActiveStage* retVal = result.Ok;
            return new ActiveStage(retVal);
        }
    }

    /// <exception cref="IronRdpException"></exception>
    /// <returns>
    /// A <c>ActiveStageOutputIterator</c> allocated on Rust side.
    /// </returns>
    public ActiveStageOutputIterator Process(DecodedImage image, Action action, byte[] payload)
    {
        unsafe
        {
            if (_inner == null)
            {
                throw new ObjectDisposedException("ActiveStage");
            }
            nuint payloadLength = (nuint)payload.Length;
            Raw.DecodedImage* imageRaw;
            imageRaw = image.AsFFI();
            if (imageRaw == null)
            {
                throw new ObjectDisposedException("DecodedImage");
            }
            Raw.Action* actionRaw;
            actionRaw = action.AsFFI();
            if (actionRaw == null)
            {
                throw new ObjectDisposedException("Action");
            }
            fixed (byte* payloadPtr = payload)
            {
                Raw.SessionFfiResultBoxActiveStageOutputIteratorBoxIronRdpError result = Raw.ActiveStage.Process(_inner, imageRaw, actionRaw, payloadPtr, payloadLength);
                if (!result.isOk)
                {
                    throw new IronRdpException(new IronRdpError(result.Err));
                }
                Raw.ActiveStageOutputIterator* retVal = result.Ok;
                return new ActiveStageOutputIterator(retVal);
            }
        }
    }

    /// <summary>
    /// Returns the underlying raw handle.
    /// </summary>
    public unsafe Raw.ActiveStage* AsFFI()
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

            Raw.ActiveStage.Destroy(_inner);
            _inner = null;

            GC.SuppressFinalize(this);
        }
    }

    ~ActiveStage()
    {
        Dispose();
    }
}
