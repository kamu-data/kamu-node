import { Component } from '@angular/core';
import {environment} from "../../../environments/environment";


@Component({
  selector: 'app-login',
  templateUrl: './login.component.html',
  styleUrls: ['./login.component.sass']
})
export class LoginComponent {

  public githubUrl: string = `https://github.com/login/oauth/authorize?scope=user:email&client_id=${environment.github_client_id}`;

}
