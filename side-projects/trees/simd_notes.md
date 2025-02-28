https://www.youtube.com/watch?v=qejTqnxQRcw

#### Naming Conventions
ps : packed single precision
ph : packed half
pd : packed double
pch : packed half-precision complex  ? 
pi8 : int 8, for 64 bit simd register
pi16 : int 16, for 64 bit simd register
pi32 : int 32, for 64 bit simd register
epi8 : int 8, extended for 128 bit simd register
epi16 : int 16, extended


#### Register Names
xmm0-xmm15 - original sse 128 bit registers  
ymm0-ymm15 - 16 256 bit registers implmemented with avx  
zmm0-ymm15 - 16 512 bit registers implemented with avx-512  

the vector types corresponding to these are   
__m128, __m128d, and __m128i for single precision floats, double float, and various ints.  
__m256, __m256d, __m512i follow this pattern.   

#### Instruction Reference
https://en.algorithmica.org/hpc/simd/intrinsics/
< size, action, type >
_mm_add_epi16 : add two 128 bit registers of 16 bit extended packed integers.
_mm256_acos_pd : acos 4 64 bit floats
_mm256_broadcast_sd : broadcast a single double to a simd vec (splat)
_mm256_ceil_pd: round 4 doubles up to an int
_mm256_cmpeq_epi32: compare 2 packed ints and return a mask of ones for eq element pairs.
__mm256_blendv_ps : pick elements from 1 of 2 vectors according to a mask


```cpp
    ///fill a 512 bit register with ints
    load_value(int32_t fill)
    {
        return _mm512_set1_epi32(i);
    }

    ///load into a 512 bit register from a slice of 16 32 bit floats
    load_from(float const* psrc);
    {
        _mm512_loadu_ps(psrc); 
    }

    ///load into a 512 bit register from a slice of 16 32 bit floats
    load_from(float const* psrc);
    {
        _mm512_loadu_ps(psrc);
    }
```