// #[derive(Debug)]
// enum X{
//     A = 0,
//     B = 1,
//     C = 2,
// }

// #[derive(Debug)]
// enum Y{
//     A = 0,
//     B = 1,
//     C = 2,
//     SA = 3,
//     SB = 4,
//     SC = 5,
// }

// impl X{
//     #[inline(never)]
//     pub fn conv(&self) -> usize{
//         match self{
//             X::A => 0,
//             X::B=>1,
//             X::C=>2,
//         }
//     }

//     #[inline(never)]
//     pub fn conv_y(self) -> Y{
//         match self{
//             X::A => Y::A,
//             X::B => Y::B,
//             X::C => Y::C,
//         }
//     }

//     pub fn from_us(s : usize) -> Self{
//         match s{
//             0 => X::A,
//             1 => X::B,
//             2 => X::C,
//             _ => unsafe{core::hint::unreachable_unchecked()}
//         }
//     }
// }

// impl Y{
//     #[inline(never)]
//     pub fn conv(&self) -> X{
//         match self{
//             Y::A => X::A,
//             Y::B=> X::B,
//             Y::C=> X::C,
//             Y::SA => X::A,
//             Y::SB => X::B,
//             Y::SC => X::C,
//         }
//     }

//     pub fn conv_us(&self) -> usize{
//         match self{
//             Y::A => 0,
//             Y::B=> 1,
//             Y::C=> 2,
//             Y::SA => 3,
//             Y::SB => 4,
//             Y::SC => 5,
//         }
//     }

//     #[inline(never)]
//     pub fn conv2(&self) -> X{
//         let mut  n = self.conv_us();
//         if n > 3{
//             n -=3;
//         }
//         X::from_us(n)
//     }
// }

// fn main(){
//     let x = X::A;
//     let a = x.conv();
//     println!("{}",a);
//     println!("{:?}",x.conv_y());
//     println!("{:?}",X::B.conv_y().conv());
//     println!("{:?}",X::B.conv_y().conv2());
// }
