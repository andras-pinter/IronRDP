// <auto-generated/> by Diplomat

#pragma warning disable 0105
using System;
using System.Runtime.InteropServices;

using Devolutions.IronRdp.Diplomat;
#pragma warning restore 0105

namespace Devolutions.IronRdp.Raw;

#nullable enable

[StructLayout(LayoutKind.Sequential)]
public partial struct WriteBuf
{
    private const string NativeLib = "DevolutionsIronRdp";

    [DllImport(NativeLib, CallingConvention = CallingConvention.Cdecl, EntryPoint = "WriteBuf_new", ExactSpelling = true)]
    public static unsafe extern WriteBuf* New();

    [DllImport(NativeLib, CallingConvention = CallingConvention.Cdecl, EntryPoint = "WriteBuf_clear", ExactSpelling = true)]
    public static unsafe extern void Clear(WriteBuf* self);

    [DllImport(NativeLib, CallingConvention = CallingConvention.Cdecl, EntryPoint = "WriteBuf_read_into_buf", ExactSpelling = true)]
    public static unsafe extern PduFfiResultVoidBoxIronRdpError ReadIntoBuf(WriteBuf* self, byte* buf, nuint bufSz);

    [DllImport(NativeLib, CallingConvention = CallingConvention.Cdecl, EntryPoint = "WriteBuf_destroy", ExactSpelling = true)]
    public static unsafe extern void Destroy(WriteBuf* self);
}
